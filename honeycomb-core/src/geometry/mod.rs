//! geometry representation types & operators
//!
//! This module contains all code related to custom spatial reprentation type implementations.
//! This include custom vector / vertex types as well as generic traits for value encoding.

// ------ MODULE DECLARATIONS

mod dim2;
mod dim3;

// ------ IMPORTS

use std::fmt::Debug;
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

// ------ CONTENT

// --- re-exports

pub use dim2::{vector::Vector2, vertex::Vertex2};
pub use dim3::{vector::Vector3, vertex::Vertex3};

// --- error enum

/// Coordinates-level error enum
#[derive(Debug, PartialEq)]
pub enum CoordsError {
    /// Error during the computation of the unit direction vector.
    ///
    /// This is returned when trying to compute the unit vector of a null [`Vector2`].
    InvalidUnitDir,
    /// Error during the computation of the normal direction vector.
    ///
    /// This is returned when trying to compute the normal to a null [`Vector2`].
    InvalidNormDir,
}

// --- generic fp repersentation trait

/// Common trait implemented by types used for coordinate representation.
///
/// The static lifetime is a requirements induced by specific implementations that use
/// [`TypeId`][std::any::TypeId];
/// This is used in order to identify types in two contexts:
/// - Interacting with VTK files (`io` feature),
/// - Coding vertices and generic attributes handling
pub trait CoordsFloat:
    num_traits::Float + Default + AddAssign + SubAssign + MulAssign + DivAssign + Debug + 'static
{
}

impl<
        T: num_traits::Float
            + Default
            + AddAssign
            + SubAssign
            + MulAssign
            + DivAssign
            + Debug
            + 'static,
    > CoordsFloat for T
{
}
