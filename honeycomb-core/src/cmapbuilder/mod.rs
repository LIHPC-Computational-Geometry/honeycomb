//! Builder implementation for combinatorial map structures.

// ------ MODULE DECLARATIONS

#[cfg(feature = "utils")]
mod grid;
#[cfg(feature = "io")]
mod io;
mod structure;

// ------ RE-EXPORTS

#[cfg(feature = "utils")]
pub use grid::GridDescriptor;
pub use structure::{BuilderError, CMapBuilder};

// ------ CONTENT

// ------ TESTS
#[cfg(test)]
mod tests;
