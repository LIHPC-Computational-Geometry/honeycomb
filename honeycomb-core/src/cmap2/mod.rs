//! [`CMap2`] code
//!
//! This module contains code for the 2D implementation of combinatorial maps: [`CMap2`].
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

mod basic_ops;
mod embed;
mod link_and_sew;
mod structure;

#[cfg(feature = "io")]
mod io;

#[cfg(feature = "utils")]
mod utils;

// ------ RE-EXPORTS

pub use structure::CMap2;

// ------ CONTENT

/// Number of beta functions defined for [`CMap2`].
const CMAP2_BETA: usize = 3;

// ------ TESTS
mod advanced_ops;
#[cfg(test)]
mod tests;
