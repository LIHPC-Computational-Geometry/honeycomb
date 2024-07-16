//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use std::collections::HashMap;

use crate::{grisubal::model::Geometry2, GridCellId, IsBoundary};
use honeycomb_core::{CMap2, CMapBuilder, CoordsFloat, DartIdentifier, Vertex2, VertexIdentifier};

use super::model::GeometryVertex;

// ------ CONTENT

macro_rules! left_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cy: ident) => {{
        let s = ($vdart.x() - $va.x()) / ($vb.x() - $va.x());
        ($vdart.y() - $va.y() - ($vb.y() - $va.y()) * s) / $cy
    }};
}

macro_rules! right_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cy: ident) => {{
        let s = ($vdart.x() - $va.x()) / ($vb.x() - $va.x());
        (($vb.y() - $va.y()) * s - ($vdart.y() - $va.y())) / $cy
    }};
}

macro_rules! down_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cx: ident) => {{
        let s = ($vdart.y() - $va.y()) / ($vb.y() - $va.y());
        (($vb.x() - $va.x()) * s - ($vdart.x() - $va.x())) / $cx
    }};
}

macro_rules! up_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cx: ident) => {{
        let s = ($vdart.y() - $va.y()) / ($vb.y() - $va.y());
        (($vdart.x() - $va.x()) - ($vb.x() - $va.x()) * s) / $cx
    }};
}

/// Inner building routine.
///
/// This function builds a combinatorial map from the described geometry. The returned
/// map is an adjusted grid that can be clipped in order to keep only part of the mesh.
/// See [`grisubal::Clip`] for more information.
///
/// # Arguments
///
/// - `geometry: &Geometry2<T>` -- Description of the input geometry.
///
/// ## Generics
///
/// - `T: CoordsFloat` -- Floating point type used for coordinate representation.
#[allow(clippy::too_many_lines)]
pub fn build_mesh<T: CoordsFloat>(geometry: &Geometry2<T>, grid_cell_sizes: (T, T)) -> CMap2<T> {
    // build the overlapping grid we'll modify
    let bbox = geometry.bbox();
    let (cx, cy) = grid_cell_sizes; // will need later
    let (nx, ny) = (
        (bbox.max_x / cx).ceil().to_usize().unwrap(),
        (bbox.max_y / cy).ceil().to_usize().unwrap(),
    );
    let ogrid = bbox.overlapping_grid(grid_cell_sizes);
    let mut cmap = CMapBuilder::default()
        .grid_descriptor(ogrid)
        .add_attribute::<IsBoundary>() // will be used for clipping
        .build()
        .expect("E: could not build overlapping grid map");

    // process the geometry

    // STEP 1
    // the aim of this step is to build an exhaustive list of the segments making up
    // the GEOMETRY INTERSECTED WITH THE GRID, i.e. for each segment, if both vertices
    // do not belong to the same cell, we break it into sub-segments until it is the case.

    let mut n_vertices = geometry.vertices.len();
    let mut new_segments = HashMap::with_capacity(geometry.poi.len() * 2); // that *2 has no basis
    geometry.segments.iter().for_each(|&(v1_id, v2_id)| {
        // fetch vertices of the segment
        let (v1, v2) = (&geometry.vertices[v1_id], &geometry.vertices[v2_id]);
        // compute their position in the grid
        // we assume that the origin of the grid is at (0., 0.)
        let (c1, c2) = (
            GridCellId(
                (v1.x() / cx).floor().to_usize().unwrap(),
                (v1.y() / cy).floor().to_usize().unwrap(),
            ),
            GridCellId(
                (v2.x() / cx).floor().to_usize().unwrap(),
                (v2.y() / cy).floor().to_usize().unwrap(),
            ),
        );
        // check neighbor status
        match GridCellId::man_dist(&c1, &c2) {
            // trivial case:
            // v1 & v2 belong to the same cell
            0 => {
                new_segments.insert(
                    if geometry.poi.contains(&v1_id) {
                        GeometryVertex::PoI(v1_id)
                    } else {
                        GeometryVertex::Regular(v1_id)
                    },
                    if geometry.poi.contains(&v2_id) {
                        GeometryVertex::PoI(v2_id)
                    } else {
                        GeometryVertex::Regular(v2_id)
                    },
                );
            }
            // ok case:
            // v1 & v2 belong to neighboring cells
            1 => {
                // fetch base dart the cell of v1
                let d0 = (1 + 4 * c1.0 + nx * 4 * c1.1) as DartIdentifier;
                // which edge of the cell are we intersecting?
                let diff = GridCellId::diff(&c1, &c2);
                #[rustfmt::skip]
                let (mut t, dart_id) = match diff {
                    // cross left
                    (-1,  0) => {
                        // vertex associated to crossed dart, i.e. top left corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0    ).unwrap() * cx,
                            T::from(c1.1 + 1).unwrap() * cy,
                        ));
                        // call macro
                        let t = left_intersec!(v1, v2, v_dart, cy);
                        (t, d0 + 3)
                    }
                    // cross right
                    ( 1,  0) => {
                        // vertex associated to crossed dart, i.e. down right corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0 + 1).unwrap() * cx,
                            T::from(c1.1    ).unwrap() * cy,
                        ));
                        let t = right_intersec!(v1, v2, v_dart, cy);
                        (t, d0 + 1) // adjust for dart direction
                    }
                    // cross down
                    ( 0, -1) => {
                        // vertex associated to crossed dart, i.e. down left corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0).unwrap() * cx,
                            T::from(c1.1).unwrap() * cy,
                        ));
                        let t = down_intersec!(v1, v2, v_dart, cx);
                        (t, d0)
                    }
                    // cross up
                    ( 0,  1) => {
                        // vertex associated to crossed dart, i.e. up right corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0 + 1).unwrap() * cx,
                            T::from(c1.1 + 1).unwrap() * cy,
                        ));
                        let t = up_intersec!(v1, v2, v_dart, cx);
                        (t, d0 + 2)
                    }
                    _ => unreachable!(),
                };
                // t is adjusted for dart direction; but it needs to be adjusted according to the edge's direction
                let edge_id = cmap.edge_id(dart_id);
                // works in 2D because edges are 2 darts at most
                if edge_id != dart_id {
                    t = T::one() - t;
                }
                // insert the intersection point & add it to the vertices/poi
                cmap.split_edge(edge_id, Some(t));
                let new_vid = cmap.beta::<1>(dart_id) as VertexIdentifier;
                new_segments.insert(
                    if geometry.poi.contains(&v1_id) {
                        GeometryVertex::PoI(v1_id)
                    } else {
                        GeometryVertex::Regular(v1_id)
                    },
                    GeometryVertex::Intersec(new_vid),
                );
                new_segments.insert(
                    GeometryVertex::Intersec(new_vid),
                    if geometry.poi.contains(&v2_id) {
                        GeometryVertex::PoI(v2_id)
                    } else {
                        GeometryVertex::Regular(v2_id)
                    },
                );
            }
            // highly annoying case:
            // v1 & v2 do not belong to neighboring cell
            i => {
                // because we're using strait segments (not curves), the manhattan distance gives us
                // the number of cell we're going through to reach v2 from v1, which is equal to the number of
                // additional vertices resulting from intersection with the grid
                // i.e. we're generating i+1 segments
                let move_along = *v2 - *v1;
                let mut t = T::zero();
                todo!()
            }
        };
    });

    // STEP 2

    // return result
    cmap
}

/// Clipping routine.
///
/// This function takes a map built by [`build_mesh`] and removes cells that model the "inside" of
/// the geometry.
pub fn remove_inner<T: CoordsFloat>(
    cmap2: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    invert_normal_dir: bool,
) {
    todo!()
}

/// Clipping routine
///
/// This function takes a map built by [`build_mesh`] and removes cells that model the "outside" of
/// the geometry.
pub fn remove_outer<T: CoordsFloat>(
    cmap2: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    invert_normal_dir: bool,
) {
    todo!()
}
