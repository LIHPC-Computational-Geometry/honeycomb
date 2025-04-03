//! remeshing routine components
//!
//! This module contains all the code used in usual remeshing loops, among which are:
//!
//! - vertex relaxation routines
//! - cell division routines
//! - cell fusion routines
//! - swap-based cell edition routines

mod collapse;
mod cut;
mod relaxation;
mod swap;

pub use collapse::collapse_edge;
pub use cut::{cut_inner_edge, cut_outer_edge};
pub use relaxation::move_vertex_to_average;
pub use swap::{EdgeSwapError, swap_edge};

#[cfg(test)]
mod tests;
