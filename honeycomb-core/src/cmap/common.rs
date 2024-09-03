//! Crate-level common definitions

// ------ IMPORTS

#[cfg(doc)]
use std::any::TypeId;
use std::fmt::Debug;

// ------ CONTENT

// --- darts

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

/// Null value for dart identifiers
pub const NULL_DART_ID: DartIdentifier = 0;

// --- maps

/// Map-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when operating on a map.
#[derive(Debug, PartialEq)]
pub enum CMapError {
    /// Variant used when requesting a vertex using an ID that has no associated vertex
    /// in storage.
    UndefinedVertex,
}
