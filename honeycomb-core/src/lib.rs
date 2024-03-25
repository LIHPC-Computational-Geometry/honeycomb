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
//! - `benchmarking_utils` -- provides additionnal methods for benchmarking and debugging
//! - `single_precision` -- uses `f32` instead of `f64` for coordinates representation

// ------ MODULE DECLARATIONS

mod cells;
mod embed;
mod orbits;
mod spatial_repr;
mod twomap;

// ------ RE-EXPORTS

pub use cells::{
    collections::{EdgeCollection, FaceCollection, VertexCollection},
    identifiers::*,
};
pub use embed::{SewPolicy, UnsewPolicy};
pub use orbits::{Orbit2, OrbitPolicy};
pub use spatial_repr::{
    coords::{Coords2, CoordsError},
    vector::Vector2,
    vertex::Vertex2,
    CoordsFloat, FloatType,
};
pub use twomap::{CMap2, CMapError};

// ------ IMPORTS

// ------ CONTENT

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
