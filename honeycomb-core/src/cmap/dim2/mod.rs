//! [`CMap2`] code
//!
//! This module contains code for the 2D implementation of combinatorial maps: [`CMap2`].
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

pub mod basic_ops;
pub mod embed;
pub mod links;
pub mod orbits;
pub mod serialize;
pub mod sews;
pub mod structure;
pub mod utils;

// ------ CONTENT

/// Number of beta functions defined for [`CMap2`].
const CMAP2_BETA: usize = 3;

// ------ TESTS

#[allow(unused_mut)]
#[cfg(test)]
mod tests;
