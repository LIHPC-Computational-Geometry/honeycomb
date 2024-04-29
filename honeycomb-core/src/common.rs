//! Crate-level common definitions

// ------ IMPORTS

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

// --- decimal types

cfg_if::cfg_if! {
    if #[cfg(feature = "single_precision")] {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f32;
    } else {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f64;
    }
}

/// Common trait implemented by types used for coordinate representation.
pub trait CoordsFloat:
    num::Float + Default + AddAssign + SubAssign + MulAssign + DivAssign
{
}

impl CoordsFloat for f32 {}
impl CoordsFloat for f64 {}
