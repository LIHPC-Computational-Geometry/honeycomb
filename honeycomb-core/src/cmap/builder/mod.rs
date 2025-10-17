//! Builder implementation for combinatorial map structures.

pub mod io;
pub mod structure;

pub use structure::{BuilderError, CMapBuilder};

#[cfg(test)]
mod tests;
