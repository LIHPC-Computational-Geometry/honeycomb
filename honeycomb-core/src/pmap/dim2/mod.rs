//! [`PMap2`] code
//!
//! This module contains code for the 2D implementation of combinatorial maps: [`PMap2`].
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

pub mod basic_ops;
pub mod embed;
pub mod link_and_sew;
pub mod orbits;
pub mod structure;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "utils")]
pub mod utils;

const PMAP2_BETA: usize = 3;
