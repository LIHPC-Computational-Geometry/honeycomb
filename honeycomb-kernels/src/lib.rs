//! # honeycomb-kernels
//!
//! This crate implements usual meshing algorithms using combinatorial maps as the underlying mesh
//! representation structure.
//!
//! [UG]:https://lihpc-computational-geometry.github.io/honeycomb/
//!

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]

pub mod cell_insertion;
pub mod grisubal;
pub mod remeshing;
pub mod skewness;
pub mod triangulation;
pub mod utils;
