//! # honeycomb-core
//!
//! This crate implements all basic structure and methods for
//! 2D and 3D combinatorial map modeling.
//!
//! This documentation focus on the implementation side of things and API usage, for more
//! formal information about combinatorial maps, refer to the **Definitions** section of
//! the [user guide][UG].
//!
//! [UG]:https://lihpc-computational-geometry.github.io/honeycomb/
//!
//! ## Features
//!
//! Optional features can be enabled when compiling this crate:
//!
//! - `utils` -- provides additionnal implementations for map generation, benchmarking & debugging
//! - `single_precision` -- uses `f32` instead of `f64` for coordinates representation in tests

// ------ CUSTOM LINTS

// --- enable doc_auto_cfg feature if compiling in nightly
#![cfg_attr(nightly, feature(doc_auto_cfg))]
// --- some though love for the code
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
// --- some tolerance
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]

// ------ MODULE DECLARATIONS

mod attributes;
mod cells;
mod cmap2;
mod common;
mod spatial_repr;
#[cfg(feature = "utils")]
pub mod utils;

// ------ RE-EXPORTS

// --- PUBLIC API

pub use attributes::{
    collections::{AttrCompactVec, AttrSparseVec},
    traits::{AttributeBind, AttributeUpdate},
};
pub use cells::{
    collections::{
        EdgeCollection, EdgeIdentifier, FaceCollection, FaceIdentifier, VertexCollection,
        VertexIdentifier, VolumeIdentifier, NULL_EDGE_ID, NULL_FACE_ID, NULL_VERTEX_ID,
        NULL_VOLUME_ID,
    },
    orbits::{Orbit2, OrbitPolicy},
};
pub use cmap2::CMap2;
pub use common::{CMapError, CoordsFloat, DartIdentifier, NULL_DART_ID};
pub use spatial_repr::{
    coords::{Coords2, CoordsError},
    vector::Vector2,
    vertex::Vertex2,
};
