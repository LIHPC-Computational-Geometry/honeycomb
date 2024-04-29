//! Map objects
//!
//! This module contains code for the two main structures provided
//! by the crate:
//!
//! - [`CMap2`], a 2D combinatorial map implementation
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

mod basic_ops;
mod embed;
mod link_and_sew;
mod structure;
#[cfg(any(doc, feature = "utils"))]
mod utils;

// ------ RE-EXPORTS

pub use structure::CMap2;

const CMAP2_BETA: usize = 3;

// ------ TESTS
#[cfg(test)]
mod tests;
