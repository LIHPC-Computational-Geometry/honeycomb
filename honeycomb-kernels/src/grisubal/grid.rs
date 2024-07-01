//! Grid related code
//!
//! This module contains all code related to the usage of an overlapping

// ------ IMPORTS

use honeycomb_core::{CoordsFloat, GridDescriptor};

// ------ CONTENT

/// Structure used to index the overlapping grid's cases.
///
/// Cells `(X, Y)` take value in range `(0, 0)` to `(N, M)`,
/// from left to right (X), from bottom to top (Y).
pub struct GridCellId(usize, usize);

impl GridCellId {
    /// Compute the [Manhattan distance](https://en.wikipedia.org/wiki/Taxicab_geometry) between
    /// two cells.
    pub fn man_dist(lhs: Self, rhs: Self) -> usize {
        todo!()
    }
}

/// Represent a 2-dimensional bounding box
pub struct BBox2<T: CoordsFloat> {
    pub min_x: T,
    pub max_x: T,
    pub min_y: T,
    pub max_y: T,
}

impl<T: CoordsFloat> BBox2<T> {
    /// Builds the descriptor of a grid overlapping the bounding box.
    pub fn overlapping_grid(&self, (len_x, len_y): (T, T)) -> GridDescriptor<T> {
        todo!()
    }
}
