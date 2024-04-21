//! # honeycomb-core
//!
//! This crate implements all basic structure and methods for
//! 2D and 3D combinatorial map modeling.
//!
//! This documentation focus on the implementation side of things
//! and API usage, for more formal information about combinatorial
//! maps, refer to the **Definitions** section of the user guide.
//!
//! ## Features
//!
//! Optional features can be enabled when compiling this crate:
//!
//! - `utils` -- provides additionnal methods for benchmarking and debugging
//! - `single_precision` -- uses `f32` instead of `f64` for coordinates representation in tests

// ------ CUSTOM LINTS

// --- some though love for the code
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
// --- some tolerance
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
// --- some to work through later
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::if_not_else)]
#![allow(clippy::range_plus_one)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::needless_pass_by_value)]

// ------ MODULE DECLARATIONS
mod attributes;
mod cells;
mod spatial_repr;
mod twomap;

#[cfg(feature = "utils")]
pub mod utils;

// ------ RE-EXPORTS

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
    DartIdentifier, NULL_DART_ID,
};
pub use spatial_repr::{
    coords::{Coords2, CoordsError},
    vector::Vector2,
    vertex::Vertex2,
    CoordsFloat, FloatType,
};
pub use twomap::{CMap2, CMapError};
