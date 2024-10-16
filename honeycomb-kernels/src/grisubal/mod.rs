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

pub(crate) mod clip;
pub(crate) mod grid;
pub(crate) mod kernel;
pub(crate) mod model;

// ------ RE-EXPORTS

pub use model::Clip;

// ------ IMPORTS

use crate::grisubal::clip::{clip_left, clip_right};
use crate::grisubal::model::{
    compute_overlapping_grid, detect_orientation_issue, remove_redundant_poi, Boundary, Geometry2,
};
use honeycomb_core::prelude::{CMap2, CoordsFloat};
use thiserror::Error;
use vtkio::Vtk;

// ------ CONTENT

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

/// Global timers for execution times per-section.
#[cfg(feature = "profiling")]
static mut TIMERS: [Option<std::time::Duration>; 13] = [None; 13];

/// Kernel section.
#[cfg(feature = "profiling")]
enum Section {
    ImportVTK = 0,
    BuildGeometry,
    DetectOrientation,
    ComputeOverlappingGrid,
    RemoveRedundantPoi,
    BuildMeshTot,
    BuildMeshInit,
    BuildMeshIntersecData,
    BuildMeshInsertIntersec,
    BuildMeshEdgeData,
    BuildMeshInsertEdge,
    Clip,
    Cleanup,
}

#[cfg(feature = "profiling")]
macro_rules! unsafe_time_section {
    ($inst: ident, $sec: expr) => {
        unsafe {
            TIMERS[$sec as usize] = Some($inst.elapsed());
            $inst = std::time::Instant::now();
        }
    };
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
    #[cfg(feature = "profiling")]
    let mut instant = std::time::Instant::now();

    // load geometry from file
    let geometry_vtk = match Vtk::import(file_path) {
        Ok(vtk) => vtk,
        Err(e) => panic!("E: could not open specified vtk file - {e}"),
    };

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::ImportVTK);

    // pre-processing
    let mut geometry = Geometry2::try_from(geometry_vtk)?;

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::BuildGeometry);

    detect_orientation_issue(&geometry)?;

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::DetectOrientation);

    // compute an overlapping grid & remove redundant PoIs
    let (grid_n_cells, origin) = compute_overlapping_grid(&geometry, grid_cell_sizes)?;

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::ComputeOverlappingGrid);

    remove_redundant_poi(&mut geometry, grid_cell_sizes, origin);

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::RemoveRedundantPoi);

    // build the map
    let mut cmap = kernel::build_mesh(&geometry, grid_cell_sizes, grid_n_cells, origin);

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::BuildMeshTot);

    // optional post-processing
    match clip {
        Clip::Left => clip_left(&mut cmap)?,
        Clip::Right => clip_right(&mut cmap)?,
        Clip::None => {}
    }

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::Clip);

    // remove attribute used for clipping
    cmap.remove_attribute_storage::<Boundary>();

    #[cfg(feature = "profiling")]
    unsafe {
        TIMERS[Section::Cleanup as usize] = Some(instant.elapsed());
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            TIMERS[0].unwrap().as_nanos(),
            TIMERS[1].unwrap().as_nanos(),
            TIMERS[2].unwrap().as_nanos(),
            TIMERS[3].unwrap().as_nanos(),
            TIMERS[4].unwrap().as_nanos(),
            TIMERS[5].unwrap().as_nanos(),
            TIMERS[6].unwrap().as_nanos(),
            TIMERS[7].unwrap().as_nanos(),
            TIMERS[8].unwrap().as_nanos(),
            TIMERS[9].unwrap().as_nanos(),
            TIMERS[10].unwrap().as_nanos(),
            TIMERS[11].unwrap().as_nanos(),
            TIMERS[12].unwrap().as_nanos(),
        );
    }

    Ok(cmap)
}

// ------ TESTS

#[cfg(test)]
mod tests;
