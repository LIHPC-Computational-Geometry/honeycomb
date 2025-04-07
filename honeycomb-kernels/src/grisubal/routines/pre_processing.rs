//! Step 0 implementation

use std::collections::HashSet;

use honeycomb_core::geometry::{CoordsFloat, Vertex2};

use crate::grisubal::{
    GrisubalError,
    model::{Geometry2, GridCellId},
};

/// Check for orientation issue **per boundary**.
///
/// This function check for the most obvious orientation issue; given a boundary, are all segments making it up
/// oriented consistently. If it is not the case, then there is at least one of:
///
/// - a vertex being the origin of two segment
/// - a vertex being the end-point of two segment
///
/// This does not cover consistent orientation across distinct boundaries (e.g. a geometry with a hole in it).
pub fn detect_orientation_issue<T: CoordsFloat>(
    geometry: &Geometry2<T>,
) -> Result<(), GrisubalError> {
    let mut origins = HashSet::new();
    let mut endpoints = HashSet::new();

    for (orig, endp) in &geometry.segments {
        if !origins.insert(orig) || !endpoints.insert(endp) {
            return Err(GrisubalError::InconsistentOrientation(
                "in-boundary inconsistency",
            ));
        }
    }

    Ok(())
}

#[allow(clippy::cast_precision_loss)]
pub fn compute_overlapping_grid<T: CoordsFloat>(
    geometry: &Geometry2<T>,
    [len_cell_x, len_cell_y]: [T; 2],
    keep_all_poi: bool,
) -> Result<([usize; 2], Vertex2<T>), GrisubalError> {
    // compute the minimum bounding box
    let (mut min_x, mut max_x, mut min_y, mut max_y): (T, T, T, T) = {
        let Some(tmp) = geometry.vertices.first() else {
            return Err(GrisubalError::InvalidShape("no vertex in shape"));
        };

        (tmp.x(), tmp.x(), tmp.y(), tmp.y())
    };

    geometry.vertices.iter().for_each(|v| {
        min_x = min_x.min(v.x());
        max_x = max_x.max(v.x()); // may not be optimal
        min_y = min_y.min(v.y()); // don't care
        max_y = max_y.max(v.y());
    });

    if max_x <= min_x {
        return Err(GrisubalError::InvalidShape(
            "bounding values along X axis are equal",
        ));
    }
    if max_y <= min_y {
        return Err(GrisubalError::InvalidShape(
            "bounding values along Y axis are equal",
        ));
    }

    // compute characteristics of the overlapping Cartesian grid

    // create a ~one-and-a-half cell buffer to contain the geometry
    // this, along with the `+1` below, guarantees that
    // dart at the boundary of the grid are not intersected by the geometry
    let mut og_x = min_x - len_cell_x * T::from(1.5).unwrap();
    let mut og_y = min_y - len_cell_y * T::from(1.5).unwrap();
    // we check for some extremely annoying cases here
    // if some are detected, the origin is incrementally shifted
    let (mut on_corner, mut reflect) = detect_overlaps(
        geometry,
        [len_cell_x, len_cell_y],
        Vertex2(og_x, og_y),
        !keep_all_poi,
    );
    let mut i = 1;

    while on_corner | reflect {
        eprintln!(
            "W: land on corner: {on_corner} - reflect on an axis: {reflect}, shifting origin"
        );
        og_x += len_cell_x * T::from(1. / (2_i32.pow(i + 1) as f32)).unwrap();
        og_y += len_cell_y * T::from(1. / (2_i32.pow(i + 1) as f32)).unwrap();
        (on_corner, reflect) = detect_overlaps(
            geometry,
            [len_cell_x, len_cell_y],
            Vertex2(og_x, og_y),
            !keep_all_poi,
        );
        i += 1;
    }

    let n_cells_x = ((max_x - og_x) / len_cell_x).ceil().to_usize().unwrap() + 1;
    let n_cells_y = ((max_y - og_y) / len_cell_y).ceil().to_usize().unwrap() + 1;

    Ok(([n_cells_x, n_cells_y], Vertex2(og_x, og_y)))
}

/// Remove from their geometry points of interest that intersect with a grid of specified dimension.
///
/// This function works under the assumption that the grid is Cartesian & has its origin on `(0.0, 0.0)`.
pub fn remove_redundant_poi<T: CoordsFloat>(
    geometry: &mut Geometry2<T>,
    [cx, cy]: [T; 2],
    origin: Vertex2<T>,
) {
    // PoI that land on the grid create a number of issues; removing them is ok since we're intersecting the grid
    // at their coordinates, so the shape will be captured via intersection anyway
    geometry.poi.retain(|idx| {
        let v = geometry.vertices[*idx];
        // origin is assumed to be (0.0, 0.0)
        let on_x_axis = ((v.x() - origin.x()) % cx).is_zero();
        let on_y_axis = ((v.y() - origin.y()) % cy).is_zero();
        !(on_x_axis | on_y_axis)
    });
}

#[allow(clippy::map_all_any_identity)]
pub fn detect_overlaps<T: CoordsFloat>(
    geometry: &Geometry2<T>,
    [cx, cy]: [T; 2],
    origin: Vertex2<T>,
    overlap_only_corners: bool,
) -> (bool, bool) {
    let on_grid = geometry
        .vertices
        .iter()
        .map(|v| {
            let on_x_axis = ((v.x() - origin.x()) % cx).is_zero();
            let on_y_axis = ((v.y() - origin.y()) % cy).is_zero();
            if overlap_only_corners {
                on_x_axis && on_y_axis
            } else {
                on_x_axis || on_y_axis
            }
        })
        .any(|a| a);

    let bad_reflection = geometry
        .vertices
        .iter()
        .enumerate()
        .filter_map(|(id, v)| {
            let on_x_axis = ((v.x() - origin.x()) % cx).is_zero();
            let on_y_axis = ((v.y() - origin.y()) % cy).is_zero();
            if on_x_axis | on_y_axis {
                return Some(id);
            }
            None
        })
        // skip vertices that do not belong to the boundary
        .filter(|id| {
            geometry
                .segments
                .iter()
                .any(|(v1, v2)| (id == v1) || (id == v2))
        })
        .map(|id| {
            // if a vertex appear in the boundary, there should be both a segment landing and a
            // segment starting on the vertex; hence `.expect()`
            let vid_in = geometry
                .segments
                .iter()
                .find_map(|(vin, ref_id)| {
                    if id == *ref_id {
                        return Some(*vin);
                    }
                    None
                })
                .expect("E: found a vertex with no incident segment - is the geometry open?");
            // same
            let vid_out = geometry
                .segments
                .iter()
                .find_map(|(ref_id, vout)| {
                    if id == *ref_id {
                        return Some(*vout);
                    }
                    None
                })
                .expect("E: found a vertex with no incident segment - is the geometry open?");
            let v_in = geometry.vertices[vid_in];
            let v_out = geometry.vertices[vid_out];
            let Vertex2(ox, oy) = origin;
            let (c_in, c_out) = (
                GridCellId(
                    ((v_in.x() - ox) / cx).floor().to_usize().unwrap(),
                    ((v_in.y() - oy) / cy).floor().to_usize().unwrap(),
                ),
                GridCellId(
                    ((v_out.x() - ox) / cx).floor().to_usize().unwrap(),
                    ((v_out.y() - oy) / cy).floor().to_usize().unwrap(),
                ),
            );
            // if v_in and v_out belong to the same grid cell, there was a "reflection" on one
            // of the grid's axis
            c_in == c_out
        })
        .any(|a| a);

    (on_grid, bad_reflection)
}
