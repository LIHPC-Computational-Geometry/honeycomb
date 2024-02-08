//! # honeycomb-core
//!
//! This crate implements all basic structure and methods for
//! 2D and 3D combinatorial map modeling.
//!
//! This documentation focus on the implementation side of things
//! and API usage, for more formal information about combinatorial
//! maps, refer to the **Definitions** section of the user guide.

// ------ MODULE DECLARATIONS

pub mod cmap;

// ------ RE-EXPORTS

pub use cmap::dart::{Dart, DartIdentifier};

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
