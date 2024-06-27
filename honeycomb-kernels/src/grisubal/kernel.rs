//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::grisubal::inp::Geometry2;
use honeycomb_core::{CMap2, CoordsFloat};

// ------ CONTENT

pub fn build_mesh<T: CoordsFloat>(geometry: &Geometry2<T>) -> CMap2<T> {
    todo!()
}

pub fn remove_inner<T: CoordsFloat>(
    cmap2: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    invert_normal_dir: bool,
) {
    todo!()
}

pub fn remove_outer<T: CoordsFloat>(
    cmap2: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    invert_normal_dir: bool,
) {
    todo!()
}
