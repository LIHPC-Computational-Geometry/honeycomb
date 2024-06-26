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
pub(crate) mod inp;
pub(crate) mod kernel;

// ------ IMPORTS

use crate::{Clamp, Geometry2};
use honeycomb_core::{CMap2, CoordsFloat};
use vtkio::Vtk;

// ------ CONTENT

/// Main algorithm call function.
///
/// # Example
///
/// ```should_panic
/// todo!()
/// ```
pub fn grisubal<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path>,
    invert_normal_dir: bool,
    clamp: Option<Clamp>,
) -> CMap2<T> {
    // load geometry from file
    let geometry_vtk = match Vtk::import(file_path) {
        Ok(vtk) => vtk,
        Err(e) => panic!("E: could not load geometry from vtk file - {}", e),
    };
    // pre-processing
    let geometry = Geometry2::from(geometry_vtk);
    // build the map
    let mut cmap = kernel::build_mesh(&geometry);
    // optional post-processing
    match clamp.unwrap_or(Clamp::None) {
        Clamp::All => {
            kernel::remove_inner(&mut cmap, &geometry, invert_normal_dir);
            kernel::remove_outer(&mut cmap, &geometry, invert_normal_dir);
        }
        Clamp::Inner => kernel::remove_inner(&mut cmap, &geometry, invert_normal_dir),
        Clamp::Outer => kernel::remove_outer(&mut cmap, &geometry, invert_normal_dir),
        Clamp::None => {}
    }
    // return result
    cmap
}

// ------ TESTS
#[cfg(test)]
mod tests;
