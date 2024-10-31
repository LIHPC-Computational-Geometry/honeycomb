//! *GRISUBAL* algorithm description & implementation
//!
//! This module contain all code used to implement our grid submersion algorithm, or *GRISUBAL*
//! for short.
//!
//! This algorithm builds the mesh of a geometry by overlapping a grid over it and intersecting
//! the grid with the geometry. It is inspired by the approach described in
//! [this](https://internationalmeshingroundtable.com/assets/research-notes/imr32/2011.pdf)
//! research note.
//!
//! # Assumptions / Hypotheses
//!
//! Boundaries are consistently oriented, i.e.:
//! - normals of segments making up a boundary all point outward / inward, no mix
//! - boundaries are closed
//! - if there are nested boundaries, their orientation are consistent one with the other; this is
//!   an extension of the first condition
//!
//! # Algorithm
//!
//! The steps followed by the algorithm are detailed in the user guide. The following is a summary.
//!
//! ## Pre-processing
//!
//! 1. Compute characteristics of a grid covering the entire geometry, avoiding exact intersection
//!    between the grid's segments and the geometry's vertices.
//! 2. Remove "redundant" Points of Interest to avoid duplicated vertices.
//! 3. Check for obvious orientation issues (open geometry & orientation per boundary).
//!
//! ## Main kernel
//!
//! 1. Compute intersection vertices between the geometry's segments and the grid.
//! 2. Insert given intersections into the grid.
//! 3. Build new edge data by searching through the original segments.
//! 4. Insert the new edges into the map. Mark darts on each side of the edge with the `Boundary`
//!    attribute.
//!
//! ## Post-processing clip
//!
//! Depending on the specified argument, one side (or the other) of the boundary can be clipped.
//! This is specified using the [`Clip`] enum; The following steps describe the operation for
//! [`Clip::Left`].
//!
//! 1. Fetch all darts marked as `Boundary::Left` during the last step of the main kernel.
//! 2. Use these darts' faces as starting point for a coloring algorithm. The search is done using
//!    a BFS and only consider adjacent faces if the adjacent dart isn't marked as a boundary.
//!    This step is also used to check for orientation inconsistencies, most importantly orientation
//!    across distinct boundaries.
//! 3. Delete all darts making up the marked faces.
//!
//! The `Boundary` attribute is then removed from the map before return.

// ------ MODULE DECLARATIONS

pub(crate) mod kernel;
pub(crate) mod model;
pub(crate) mod timers;

// ------ IMPORTS

use crate::grisubal::{
    kernel::{
        clip_left, clip_right, compute_intersection_ids, compute_overlapping_grid,
        detect_orientation_issue, generate_edge_data, generate_intersection_data,
        group_intersections_per_edge, insert_edges_in_map, insert_intersections,
        remove_redundant_poi,
    },
    model::{Boundary, Geometry2},
    timers::{finish, start_timer, unsafe_time_section},
};
use honeycomb_core::{
    cmap::{CMapBuilder, GridDescriptor},
    prelude::{CMap2, CoordsFloat},
};
use thiserror::Error;
use vtkio::Vtk;

// ------ CONTENT

/// Post-processing clip operation.
///
/// Note that the part of the map that is clipped depends on the orientation of the original geometry provided as
/// input.
#[derive(Default)]
pub enum Clip {
    /// Clip elements located on the left side of the oriented boundary.
    Left,
    /// Clip elements located on the right side of the oriented boundary.
    Right,
    /// Keep all elements. Default value.
    #[default]
    None,
}

#[derive(Error, Debug)]
/// Enum used to model potential errors of the `grisubal` kernel.
///
/// Each variant has an associated message that details more precisely what was detected.
pub enum GrisubalError {
    /// An orientation issue has been detected in the input geometry.
    #[error("boundary isn't consistently oriented - {0}")]
    InconsistentOrientation(&'static str),
    /// The specified geometry does not match one (or more) requirements of the algorithm.
    #[error("input shape isn't conform to requirements - {0}")]
    InvalidShape(&'static str),
    /// The VTK file used to try to build a `Geometry2` object contains invalid data
    /// (per VTK's specification).
    #[error("invalid/corrupted data in the vtk file - {0}")]
    BadVtkData(&'static str),
    /// The VTK file used to try to build a `Geometry2` object contains valid but unsupported data.
    #[error("unsupported data in the vtk file - {0}")]
    UnsupportedVtkData(&'static str),
}

#[allow(clippy::missing_errors_doc)]
/// Main algorithm call function.
///
/// # Arguments
///
/// - `file_path: impl AsRef<Path>` -- Path to a VTK file describing input geometry. See
///   [VTK Format] for more information about the expected formatting.
/// - `grid_cell_sizes: [T; 2],` -- Desired grid cell size along the X/Y axes.
/// - `clip: Option<Clip>` -- Indicates which part of the map should be clipped, if any, in
///   the post-processing phase. For more information on the clipping process, see [`Clip`].
///
/// ## VTK Format
///
/// At the moment, the input geometry should be specified via a file under the VTK Legacy format.
/// Just like the `io` feature provided in the core crate, there are a few additional requirements
/// for the geometry to be loaded correctly:
/// - The geometry should have a consistent orientation, i.e. the order in which the points are
///   given should form normals with a consistent direction (either pointing inward or outward the
///   geometry).
/// - The geometry should be described using in an `UnstructuredGrid` data set, with supported
///   cell types (`Vertex`, `Line`). Lines will be interpreted as the boundary to match while
///   vertices will be considered as points of interests.
///
/// # Return / Errors
///
/// This function returns a `Result` taking the following values:
/// - `Ok(CMap2)` -- Algorithm ran successfully.
/// - `Err(GrisubalError)` -- Algorithm encountered an issue. See [`GrisubalError`] for more
///   information about possible errors.
///
/// # Panics
///
/// This function may panic if the specified file cannot be opened.
///
/// # Example
///
/// ```no_run
/// # use honeycomb_core::prelude::CMap2;
/// # use honeycomb_kernels::grisubal::*;
/// # fn main() -> Result<(), GrisubalError>{
/// let cmap: CMap2<f64> = grisubal("some/path/to/geometry.vtk", [1., 1.], Clip::default())?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn grisubal<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path>,
    grid_cell_sizes: [T; 2],
    clip: Clip,
) -> Result<CMap2<T>, GrisubalError> {
    start_timer!(instant);

    // load geometry from file
    let geometry_vtk = match Vtk::import(file_path) {
        Ok(vtk) => vtk,
        Err(e) => panic!("E: could not open specified vtk file - {e}"),
    };

    unsafe_time_section!(instant, timers::Section::ImportVTK);

    // pre-processing
    let mut geometry = Geometry2::try_from(geometry_vtk)?;

    unsafe_time_section!(instant, timers::Section::BuildGeometry);

    detect_orientation_issue(&geometry)?;

    unsafe_time_section!(instant, timers::Section::DetectOrientation);

    // compute an overlapping grid & remove redundant PoIs
    let ([nx, ny], origin) = compute_overlapping_grid(&geometry, grid_cell_sizes)?;
    let [cx, cy] = grid_cell_sizes;

    unsafe_time_section!(instant, timers::Section::ComputeOverlappingGrid);

    remove_redundant_poi(&mut geometry, grid_cell_sizes, origin);

    unsafe_time_section!(instant, timers::Section::RemoveRedundantPoi);

    // compute grid characteristics
    // build grid descriptor
    let ogrid = GridDescriptor::default()
        .n_cells_x(nx)
        .n_cells_y(ny)
        .len_per_cell_x(cx)
        .len_per_cell_y(cy)
        .origin(origin);

    start_timer!(kernel);

    // build initial map
    let mut cmap = CMapBuilder::default()
        .grid_descriptor(ogrid)
        .add_attribute::<Boundary>() // will be used for clipping
        .build()
        .expect("E: unreachable"); // urneachable because grid dims are valid

    unsafe_time_section!(instant, timers::Section::BuildMeshInit);

    // process the geometry

    // STEP 1
    // the aim of this step is to build an exhaustive list of the segments making up
    // the GEOMETRY INTERSECTED WITH THE GRID, i.e. for each segment, if both vertices
    // do not belong to the same cell, we break it into sub-segments until it is the case.

    let (new_segments, intersection_metadata) =
        generate_intersection_data(&cmap, &geometry, [nx, ny], [cx, cy], origin);

    unsafe_time_section!(instant, timers::Section::BuildMeshIntersecData);

    // STEP 1.5
    // precompute stuff to
    // - parallelize step 2
    // - make step 2 and step 3 independent from each other

    let n_intersec = intersection_metadata.len();
    let (edge_intersec, dart_slices) =
        group_intersections_per_edge(&mut cmap, intersection_metadata);
    let intersection_darts = compute_intersection_ids(n_intersec, &edge_intersec, &dart_slices);

    // STEP 2
    // insert the intersection vertices into the map & recover their encoding dart. The output Vec has consistent
    // indexing with the input Vec, meaning that indices in GeometryVertex::Intersec instances are still valid.

    insert_intersections(&mut cmap, &edge_intersec, &dart_slices);

    unsafe_time_section!(instant, timers::Section::BuildMeshInsertIntersec);

    // STEP 3
    // now that we have a list of "atomic" (non-dividable) segments, we can use it to build the actual segments that
    // will be inserted into the map. Intersections serve as anchor points for the new segments while PoI make up
    // "intermediate" points of segments.

    let edges = generate_edge_data(&cmap, &geometry, &new_segments, &intersection_darts);

    unsafe_time_section!(instant, timers::Section::BuildMeshEdgeData);

    // STEP 4
    // now that we have some segments that are directly defined between intersections, we can use some N-maps'
    // properties to easily build the geometry into the map.
    // This part relies heavily on "conventions"; the most important thing to note is that the darts in `MapEdge`
    // instances are very precisely set, and can therefore be used to create all the new connectivities.

    insert_edges_in_map(&mut cmap, &edges);

    unsafe_time_section!(instant, timers::Section::BuildMeshInsertEdge);
    unsafe_time_section!(kernel, timers::Section::BuildMeshTot);

    // optional post-processing
    match clip {
        Clip::Left => clip_left(&mut cmap)?,
        Clip::Right => clip_right(&mut cmap)?,
        Clip::None => {}
    }

    unsafe_time_section!(instant, timers::Section::Clip);

    // remove attribute used for clipping
    cmap.remove_attribute_storage::<Boundary>();

    finish!(instant);

    Ok(cmap)
}

// ------ TESTS

#[cfg(test)]
mod tests;
