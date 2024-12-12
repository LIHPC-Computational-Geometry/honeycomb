//! Builder implementation for combinatorial map structures.

// ------ MODULE DECLARATIONS

pub mod grid;
pub mod structure;

#[cfg(feature = "io")]
pub mod io;

// ------ RE-EXPORTS

pub use grid::GridDescriptor;
pub use structure::{BuilderError, CMapBuilder};

// ------ CONTENT

// ------ TESTS
#[cfg(test)]
mod tests;
