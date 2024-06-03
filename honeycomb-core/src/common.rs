//! Crate-level common definitions

// ------ IMPORTS

#[cfg(doc)]
use std::any::TypeId;
use std::fmt::Debug;
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

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

// --- generic decimal trait

/// Common trait implemented by types used for coordinate representation.
///
/// The static lifetime is a requirements induced by specific implementations that use [`TypeId`];
/// This is used in order to identify types in two contexts:
/// - Interacting with VTK files (`io` feature),
/// - Coding vertices and generic attributes handling
pub trait CoordsFloat:
    num::Float + Default + AddAssign + SubAssign + MulAssign + DivAssign + Debug + 'static
{
}

impl<T: num::Float + Default + AddAssign + SubAssign + MulAssign + DivAssign + Debug + 'static>
    CoordsFloat for T
{
}
