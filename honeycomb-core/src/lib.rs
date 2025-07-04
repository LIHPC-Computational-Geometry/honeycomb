//! # honeycomb-core
//!
//! This crate implements all basic structures and methods for
//! 2D and 3D combinatorial map modeling.
//!
//! This documentation focus on the implementation side of things and API usage, for more
//! formal information about combinatorial maps, refer to the **Definitions** section of
//! the [user guide][UG].
//!
//! ## Features
//!
//! The `par-internals` feature can be enabled so that `CMap` structures use `rayon` internally
//! to accelerate some methods (e.g. `par_extend` for new element additions). This may also lead
//! to changes in performance due to first-touch mechanisms (to be confirmed).
//!
//! [UG]:https://lihpc-computational-geometry.github.io/honeycomb/

// ------ CUSTOM LINTS

// --- enable doc_auto_cfg feature if compiling in nightly
#![allow(unexpected_cfgs)]
#![cfg_attr(nightly, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]

// ------ PUBLIC API

pub mod attributes;

pub mod cmap;

pub mod geometry;

// re-export since we use their items in the API
pub use fast_stm as stm;
