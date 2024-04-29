//! Map objects
//!
//! This module contains code for the two main structures provided
//! by the crate:
//!
//! - [`CMap2`], a 2D combinatorial map implementation
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

mod basic_ops;
mod embed;
mod link_and_sew;
mod structure;
#[cfg(any(doc, feature = "utils"))]
mod utils;

// ------ RE-EXPORTS

pub use structure::CMap2;

// ------ CONTENT

/// Map-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when operating on a map.
#[derive(Debug, PartialEq)]
pub enum CMapError {
    /// Variant used when requesting a vertex using an ID that has no associated vertex
    /// in storage.
    UndefinedVertex,
}

const CMAP2_BETA: usize = 3;

// ------ TESTS
#[cfg(test)]
mod tests;
