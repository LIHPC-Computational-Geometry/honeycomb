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

// ------ MODULE DECLARATIONS

mod cells;
mod dart;
mod embed;
mod spatial_repr;
mod twomap;

#[cfg(feature = "utils")]
pub mod utils;

// ------ RE-EXPORTS

pub use cells::{
    attribute_collections::{AttrCompactVec, AttrSparseVec},
    attributes::{AttributeBind, AttributeUpdate},
    collections::{EdgeCollection, FaceCollection, VertexCollection},
    identifiers::*,
    orbits::{Orbit2, OrbitPolicy},
};
pub use dart::DartData;
pub use embed::{Face, SewPolicy, UnsewPolicy};
pub use spatial_repr::{Coords2, CoordsError, CoordsFloat, FloatType, Vector2, Vertex2};
pub use twomap::{CMap2, CMapError};
