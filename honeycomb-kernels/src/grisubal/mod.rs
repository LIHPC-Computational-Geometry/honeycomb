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
//! Let `S` be the set of all segments making up the boundaries of the geometry.
//! Let `PoI` be the set of points of interests of the boundaries (i.e. vertices that must be inserted into the final mesh)
//! Let `DoI` be the corresponding set of darts of interests
//!
//! For all segments `[A, B]` of `S`:
//! - compute the Manhattan distance `d` between cell(A) and cell(B):
//!     - if  `d == 0`: `A` and `B` belong to the same grid cell
//!         - Do nothing
//!     - if `d == 1`: `A` and `B` belong to neighbor cells
//!         - Compute the intersection `C` between the segment and the grid's edge
//!         - Split the grid's edge to add `C` on it, add relevant darts to `DoI`
//!         - Replace segment `[A, B]` by segments `[A, C]`, `[C, B]`
//!         - Add `C` to `PoI`
//!     - if `d > 1`: `A` and `B` belong to different, non-neighbor cells
//!         - Compute all intersections `Ci` between the segment and grid's edges
//!         - Split grid's edges to add `Ci`s on them, add relevant darts to `DoI`
//!         - Replace segment `[A, B]` by segments `[A, C1]`, `[C1, C2]`, ..., `[CX, B]`
//!         - Add all `Ci`s to `PoI`
//! - if `B` belongs to `PoI`:
//!     - Insert `B` into the map
//!     - Add relevant darts to `DoI`
//!
//! For all points `P` of `PoI`:
//! - search `S` to find `P'`, the first "next" point to belong to `PoI`
//! - use `DoI` to build the `[P, P']` segment into the map (this may need some refinement to avoid execution-path inconsistencies)
//!
//!
//! ## Post-processing clip
//!
//! TBD

// ------ MODULE DECLARATIONS

pub(crate) mod grid;
pub(crate) mod kernel;
pub(crate) mod model;

// ------ IMPORTS

use crate::{Clip, Geometry2};
use honeycomb_core::{CMap2, CoordsFloat};
use vtkio::Vtk;

// ------ CONTENT

/// Main algorithm call function.
///
/// # Arguments
///
/// - `file_path: impl AsRef<Path>` -- Path to a VTK file describing input geometry. See
///   [VTK Format] for more information about the expected formatting.
/// - `invert_normal_dir: bool` -- Indicates whether segments' normals point inward or outward
///   relative to the geometry.
/// - `clip: Option<Clip>` -- Indicates which part of the map should be clipped, jf any, in
///   the post-processing phase.
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
/// # Panics
///
/// This function may panic if:
/// - the specified file cannot be opened
/// - an internal routine panics, i.e.:
///     - TODO: complete
///
/// # Example
///
/// ```should_panic
/// # fn main() {
/// use honeycomb_core::CMap2;
/// use honeycomb_kernels::{Clip, grisubal};
/// // this panics because the file does not exist, but the usage is correct
/// let cmap: CMap2<f64> = grisubal("some/path/to/geometry.vtk", true, (1., 1.), Some(Clip::Outer));
/// # }
/// ```
pub fn grisubal<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path>,
    invert_normal_dir: bool,
    grid_cell_sizes: (T, T),
    clip: Option<Clip>,
) -> CMap2<T> {
    // load geometry from file
    let geometry_vtk = match Vtk::import(file_path) {
        Ok(vtk) => vtk,
        Err(e) => panic!("E: could not open specified vtk file - {e}"),
    };
    // pre-processing
    let geometry = Geometry2::from(geometry_vtk);
    // build the map
    let mut cmap = kernel::build_mesh(&geometry, grid_cell_sizes);
    // optional post-processing
    match clip.unwrap_or(Clip::None) {
        Clip::All => {
            kernel::remove_inner(&mut cmap, &geometry, invert_normal_dir);
            kernel::remove_outer(&mut cmap, &geometry, invert_normal_dir);
        }
        Clip::Inner => kernel::remove_inner(&mut cmap, &geometry, invert_normal_dir),
        Clip::Outer => kernel::remove_outer(&mut cmap, &geometry, invert_normal_dir),
        Clip::None => {}
    }
    // return result
    cmap
}

// ------ TESTS
#[cfg(test)]
mod tests;