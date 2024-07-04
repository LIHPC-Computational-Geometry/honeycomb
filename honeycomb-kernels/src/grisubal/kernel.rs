//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::grisubal::inp::Geometry2;
use honeycomb_core::{CMap2, CoordsFloat};

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
    todo!()
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
