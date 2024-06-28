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
//! ```text
//! todo
//! ```

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
/// ```
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
