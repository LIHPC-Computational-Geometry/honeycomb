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
        lhs.0.abs_diff(rhs.0) + lhs.1.abs_diff(rhs.1)
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
    pub fn overlapping_grid(&self, (len_cell_x, len_cell_y): (T, T)) -> GridDescriptor<T> {
        assert!(
            self.min_x > T::zero(),
            "E: the geometry should be entirely defined in positive Xs/Ys"
        );
        assert!(
            self.min_y > T::zero(),
            "E: the geometry should be entirely defined in positive Xs/Ys"
        );
        assert!(self.max_x > self.min_x);
        assert!(self.max_y > self.min_y);
        let n_cells_x = (self.max_x / len_cell_x).ceil().to_usize();
        let n_cells_y = (self.max_y / len_cell_y).ceil().to_usize();
        GridDescriptor::default()
            .n_cells_x(n_cells_x.unwrap())
            .n_cells_y(n_cells_y.unwrap())
            .len_per_cell_x(len_cell_x)
            .len_per_cell_y(len_cell_y)
    }
}
