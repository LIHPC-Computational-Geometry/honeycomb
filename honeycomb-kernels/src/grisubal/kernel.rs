//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{grisubal::model::Geometry2, GridCellId, IsBoundary, Segment};
use honeycomb_core::{CMap2, CMapBuilder, CoordsFloat};

// ------ CONTENT

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
pub fn build_mesh<T: CoordsFloat>(geometry: &Geometry2<T>, grid_cell_sizes: (T, T)) -> CMap2<T> {
    // build the overlapping grid we'll modify
    let bbox = geometry.bbox();
    let (cx, cy) = grid_cell_sizes; // will need later
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

    let mut new_vertices = geometry.vertices.clone();
    let mut new_poi = geometry.poi.clone();
    let new_segments = geometry.segments.iter().flat_map(|seg| {
        // fetch vertices of the segment
        let (v1, v2) = (&geometry.vertices[seg.0], &geometry.vertices[seg.1]);
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
            0 => todo!(),
            // ok case:
            // v1 & v2 belong to neighboring cells
            1 => {
                // which edge of the cell are we intersecting?

                // compute the intersection point & add it to the vertices/poi

                // return new sub-segments
                todo!()
            }
            // highly annoying case:
            // v1 & v2 do not belong to neighboring cell
            i => {
                // because we're using strait segments (not curves), the manhattan distance gives us
                // the number of cell we're going through to reach v2 from v1, which is equal to the number of
                // additional vertices resulting from intersection with the grid
                // i.e. we're generating i+1 segments
                todo!()
            }
        }
        todo!()
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
