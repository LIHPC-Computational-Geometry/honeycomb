//! Spatial representation types & operators
//!
//! This module contains all code related to custom spatial reprentation type implementations.
//! This include custom vector / vertex types as well as generic traits for value encoding.

// ------ MODULE DECLARATIONS

mod vector;
mod vertex;

// ------ IMPORTS

use std::fmt::Debug;
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

// ------ CONTENT

// --- re-exports

pub use vector::Vector2;
pub use vertex::Vertex2;

// --- error enum

/// Coordinates-level error enum
#[derive(Debug, PartialEq)]
pub enum CoordsError {
    /// Error during the computation of the unit directional vector.
    ///
    /// This is returned when trying to compute the unit vector of a null [`vector::Vector2`].
    InvalidUnitDir,
}

// --- generic fp repersentation trait

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

// ------ TESTS
#[cfg(test)]
mod tests;
