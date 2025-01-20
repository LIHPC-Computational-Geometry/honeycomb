//! geometry representation types & operators
//!
//! This module contains all code related to custom spatial representation type implementations.
//! This include custom vector / vertex types as well as a generic FP trait.

mod dim2;
mod dim3;

use std::fmt::Debug;
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};
use thiserror::Error;

//

pub use dim2::{vector::Vector2, vertex::Vertex2};
pub use dim3::{vector::Vector3, vertex::Vertex3};

/// # Coordinates-level error enum
#[derive(Error, Debug, PartialEq)]
pub enum CoordsError {
    /// Error returned when trying to compute the unit vector of a null [`Vector2`].
    #[error("cannot compute unit direction of a null vector")]
    InvalidUnitDir,
    /// Error returned when trying to compute the normal to a null [`Vector2`].
    #[error("cannot compute normal direction to a null vector")]
    InvalidNormDir,
}

/// # Generic FP type trait
///
/// This trait is used for vertex & vector values. The static lifetime is a requirements induced
/// by the attribute system implementation.
pub trait CoordsFloat:
    num_traits::Float
    + Default
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + Debug
    + Send
    + Sync
    + 'static
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
            + Send
            + Sync
            + 'static,
    > CoordsFloat for T
{
}
