//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use honeycomb_core::{CoordsFloat, GridDescriptor};

// ------ CONTENT

pub struct GridCellId(usize, usize);

impl GridCellId {
    pub fn man_dist(lhs: Self, rhs: Self) -> usize {
        todo!()
    }
}

pub struct BBox2<T: CoordsFloat> {
    pub tleft: T,
    pub tright: T,
    pub bleft: T,
    pub bright: T,
}

impl<T: CoordsFloat> BBox2<T> {
    pub fn to_grid_desc(&self, (len_x, len_y): (T, T)) -> GridDescriptor<T> {
        todo!()
    }
}
