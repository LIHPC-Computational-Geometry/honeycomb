//! # honeycomb-core
//!
//! This crate implements all basic structure and methods for
//! 2D and 3D combinatorial map modeling.
//!
//! This documentation focus on the implementation side of things
//! and API usage, for more formal information about combinatorial
//! maps, refer to the **Definitions** section of the user guide.

// ------ MODULE DECLARATIONS

pub mod dart;
pub mod embed;
pub mod twomap;

// ------ RE-EXPORTS

pub use dart::{DartIdentifier, NULL_DART_ID};
pub use embed::{
    FaceIdentifier, SewPolicy, UnsewPolicy, Vertex2, VertexIdentifier, VolumeIdentifier,
};
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
