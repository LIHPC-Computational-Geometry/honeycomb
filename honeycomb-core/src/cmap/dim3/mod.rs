//! [`CMap3`] code
//!
//! This module contains code for the 3D implementation of combinatorial maps: [`CMap3`].
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

pub mod basic_ops;
pub mod embed;
pub mod links;
pub mod orbits;
// pub mod sews;
pub mod structure;

// #[cfg(feature = "io")]
// pub mod io;

#[cfg(feature = "utils")]
pub mod utils;

/// Number of beta functions defined for [`CMap3`].
const CMAP3_BETA: usize = 4;

// -- tests

#[cfg(test)]
mod tests;
