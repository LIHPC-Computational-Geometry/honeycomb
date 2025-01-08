//! # honeycomb-core
//!
//! This crate implements all basic structures and methods for
//! 2D and 3D combinatorial map modeling.
//!
//! This documentation focus on the implementation side of things and API usage, for more
//! formal information about combinatorial maps, refer to the **Definitions** section of
//! the [user guide][UG].
//!
//! [UG]:https://lihpc-computational-geometry.github.io/honeycomb/
//!

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
pub use stm;

/// commonly used items
pub mod prelude;
