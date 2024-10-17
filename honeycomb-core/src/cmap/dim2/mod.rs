//! [`CMap2`] code
//!
//! This module contains code for the 2D implementation of combinatorial maps: [`CMap2`].
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

use crate::cmap::NULL_DART_ID;
use std::sync::atomic::AtomicU32;

pub mod basic_ops;
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

// this is ok because we use this const as a copy init value, we don't access and edit it
#[allow(clippy::declare_interior_mutable_const)]
const CMAP2_NULL_ENTRY: [AtomicU32; 3] = [
    AtomicU32::new(NULL_DART_ID),
    AtomicU32::new(NULL_DART_ID),
    AtomicU32::new(NULL_DART_ID),
];

// ------ TESTS
#[cfg(test)]
mod tests;
