//! *GRISUBAL* algorithm description & implementation
//!
//! This module contain all code used to implement our grid submersion algorithm, or *grisubal*
//! for short.
//!
//! This algorithm builds the mesh of a geometry by overlapping a grid over it and making
//! adjustment to fit the grid to the geometry. It is inspired by the approach described in
//! [this](https://internationalmeshingroundtable.com/assets/research-notes/imr32/2011.pdf)
//! research note.
//!
//! # Assumptions / Hypotheses
//!
//! - All components of the geometry are located in positive X/Y
//! - Edges are consistently oriented (i.e. normals of edges making up a face all point
//!   outward / inward, no mix)
//!
//! # Pseudo code
//!
//! ## Map generation
//!
//! **Input**:
//! - Geometry of interest
//! - Grid characteristics (length of cell along X/Y-axis)
//!
//! **Algorithm**:
//!
//! TODO
//!
//! ## Post-processing clip
//!
//! TBD

// ------ MODULE DECLARATIONS

pub(crate) mod grid;
pub(crate) mod kernel;
pub(crate) mod model;

// ------ IMPORTS

use crate::grisubal::clip::{clip_left, clip_right};
use crate::{detect_orientation_issue, remove_redundant_poi, Clip, Geometry2};
use honeycomb_core::{CMap2, CoordsFloat};
use vtkio::Vtk;
// ------ CONTENT

#[derive(Debug)]
/// Enum used to model potential errors of the `grisubal` kernel.
///
/// Each variant has an associated message that details more precisely what was detected.
pub enum GrisubalError {
    /// An orientation issue has been detected in the input geometry.
    InconsistentOrientation(String),
    /// The specified geometry does not match one (or more) requirements of the algorithm.
    InvalidInput(String),
    /// The VTK file used to try to build a `Geometry2` object contains invalid data
    /// (per VTK's specification).
    BadVtkData(&'static str),
    /// The VTK file used to try to build a `Geometry2` object contains valid but unsupported data.
    UnsupportedVtkData(&'static str),
}

#[allow(clippy::missing_errors_doc)]
/// Main algorithm call function.
///
/// # Arguments
///
/// - `file_path: impl AsRef<Path>` -- Path to a VTK file describing input geometry. See
///   [VTK Format] for more information about the expected formatting.
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
///   cell types (`Vertex`, `PolyVertex`?, `Line`, `PolyLine`?). Lines will be interpreted as the
///   geometry to match while vertices will be considered as points of interests.
///
/// # Return / Errors
///
/// TODO: complete
///
/// # Panics
///
/// This function may panic if:
/// - the specified file cannot be opened
/// - an internal routine panics, i.e.:
///     - TODO: complete
///
/// # Example
///
/// ```no_run
/// # use honeycomb_core::CMap2;
/// # use honeycomb_kernels::{grisubal, Clip, GrisubalError};
/// # fn main() -> Result<(), GrisubalError>{
/// let cmap: CMap2<f64> = grisubal("some/path/to/geometry.vtk", [1., 1.], Some(Clip::Left))?;
/// # Ok(())
/// # }
/// ```
pub fn grisubal<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path>,
    grid_cell_sizes: [T; 2],
    clip: Option<Clip>,
) -> Result<CMap2<T>, GrisubalError> {
    // load geometry from file
    let geometry_vtk = match Vtk::import(file_path) {
        Ok(vtk) => vtk,
        Err(e) => panic!("E: could not open specified vtk file - {e}"),
    };

    // pre-processing
    let mut geometry = Geometry2::try_from(geometry_vtk)?;
    detect_orientation_issue(&geometry)?;
    remove_redundant_poi(&mut geometry, grid_cell_sizes);

    // build the map
    #[allow(unused)]
    let mut cmap = kernel::build_mesh(&mut geometry, grid_cell_sizes)?;
    // optional post-processing
    match clip.unwrap_or_default() {
        Clip::Left => clip_left(cmap),
        Clip::Right => clip_right(cmap),
        Clip::None => Ok(cmap),
    }
}

// ------ TESTS
mod clip;
#[cfg(test)]
mod tests;
