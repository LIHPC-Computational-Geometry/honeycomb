//! Builder implementation for combinatorial map structures.

// ------ MODULE DECLARATIONS

pub mod grid;
pub mod io;
pub mod structure;

// ------ RE-EXPORTS

pub use grid::GridDescriptor;
pub use structure::{BuilderError, CMapBuilder};

// ------ CONTENT

// ------ TESTS
#[cfg(test)]
mod tests;
