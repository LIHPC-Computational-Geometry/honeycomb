//! Builder implementation for combinatorial map structures.

pub mod grid;
pub mod io;
pub mod structure;

pub use grid::GridDescriptor;
pub use structure::{BuilderError, CMapBuilder};

#[cfg(test)]
mod tests;
