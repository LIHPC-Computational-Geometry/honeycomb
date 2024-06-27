//! # honeycomb-kernels
//!
//! This crate implements usual meshing algorithms using combinatorial maps as the underlying mesh
//! representation structure. The implementation of those are provided in the core crate of the
//! `honeycomb` [project][UG].
//!
//! [UG]:https://lihpc-computational-geometry.github.io/honeycomb/
//!
//! ## Kernels
//!
//! - `grisubal` -- Grid Submersion Algorithm

// ------ MODULE DECLARATIONS

mod grisubal;

// ------ RE-EXPORTS

// --- PUBLIC API

pub use grisubal::grisubal;

// --- INTERNALS

pub(crate) use grisubal::grid::{BBox2, GridCellId};
pub(crate) use grisubal::inp::{Clamp, Geometry2, Segment};
