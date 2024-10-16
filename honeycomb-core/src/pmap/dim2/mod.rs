//! [`PMap2`] code
//!
//! This module contains code for the 2D implementation of combinatorial maps: [`PMap2`].
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

mod basic_ops;
mod embed;
mod link_and_sew;
mod orbits;
mod structure;

#[cfg(feature = "io")]
mod io;

#[cfg(feature = "utils")]
mod utils;

const PMAP2_BETA: usize = 3;
