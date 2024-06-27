//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

pub mod grid;
pub mod inp;
pub mod kernel;

// ------ IMPORTS

use crate::{Clamp, Geometry2};
use honeycomb_core::{CMap2, CoordsFloat};
use vtkio::Vtk;

// ------ CONTENT

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
