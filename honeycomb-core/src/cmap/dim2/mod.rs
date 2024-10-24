//! [`CMap2`] code
//!
//! This module contains code for the 2D implementation of combinatorial maps: [`CMap2`].
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

pub mod basic_ops;
#[allow(dead_code)]
pub mod components;
pub mod embed;
pub mod link_and_sew;
pub mod orbits;
pub mod structure;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "utils")]
pub mod utils;

// ------ CONTENT

/// Number of beta functions defined for [`CMap2`].
const CMAP2_BETA: usize = 3;

// ------ TESTS
#[cfg(test)]
mod tests;
