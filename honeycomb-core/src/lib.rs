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

mod coords;
pub mod dart;
pub mod embed;
mod orbits;
pub mod twomap;

// ------ RE-EXPORTS

pub use coords::{Coords2, CoordsError, CoordsFloat, FloatType};
pub use dart::{DartIdentifier, NULL_DART_ID};
pub use embed::{
    FaceIdentifier, SewPolicy, UnsewPolicy, Vertex2, VertexIdentifier, VolumeIdentifier,
};
pub use orbits::{Orbit, OrbitPolicy};
pub use twomap::TwoMap;

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
