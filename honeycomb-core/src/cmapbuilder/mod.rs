//! Builder implementation for combinatorial map structures.

// ------ MODULE DECLARATIONS

#[cfg(feature = "utils")]
pub mod grid;
#[cfg(feature = "io")]
pub mod io;
pub mod structure;

// ------ RE-EXPORTS

#[cfg(feature = "utils")]
pub use grid::descriptor::GridDescriptor;
pub use structure::{BuilderError, CMapBuilder};

// ------ CONTENT

// ------ TESTS
#[cfg(test)]
mod tests;
