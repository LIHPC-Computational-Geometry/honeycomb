//! Grid related code
//!
//! This module contains all code related to the usage of an overlapping

// ------ IMPORTS

// ------ CONTENT

/// Structure used to index the overlapping grid's cases.
///
/// Cells `(X, Y)` take value in range `(0, 0)` to `(N, M)`,
/// from left to right (X), from bottom to top (Y).
pub struct GridCellId(pub usize, pub usize);

impl GridCellId {
    /// Compute the [Manhattan distance](https://en.wikipedia.org/wiki/Taxicab_geometry) between
    /// two cells.
    pub fn man_dist(lhs: &Self, rhs: &Self) -> usize {
        lhs.0.abs_diff(rhs.0) + lhs.1.abs_diff(rhs.1)
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn diff(lhs: &Self, rhs: &Self) -> (isize, isize) {
        (
            rhs.0 as isize - lhs.0 as isize,
            rhs.1 as isize - lhs.1 as isize,
        )
    }
}
