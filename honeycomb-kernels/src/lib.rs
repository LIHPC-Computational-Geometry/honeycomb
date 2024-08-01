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

// ------ CUSTOM LINTS

// --- some though love for the code
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
// --- some tolerance
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]

// ------ MODULE DECLARATIONS

#[allow(unused)]
pub mod grisubal;

// ------ RE-EXPORTS

// --- PUBLIC API

pub use grisubal::{grisubal, model::Clip};

// --- INTERNALS

#[allow(unused)]
pub(crate) use grisubal::grid::GridCellId;
#[allow(unused)]
pub(crate) use grisubal::model::{
    compute_overlapping_grid, remove_redundant_poi, Geometry2, GeometryVertex, IsBoundary, MapEdge,
};
