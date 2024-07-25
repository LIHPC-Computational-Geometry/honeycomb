//! Spatial representation types & operators
//!
//! This module contains all code related to custom spatial reprentation type implementations.
//! This include custom vector / vertex types as well as generic traits for value encoding.

// ------ MODULE DECLARATIONS

pub mod vector;
pub mod vertex;

// ------ CONTENT

/// Coordinates-level error enum
#[derive(Debug, PartialEq)]
pub enum CoordsError {
    /// Error during the computation of the unit directional vector.
    ///
    /// This is returned when trying to compute the unit vector of a null [`vector::Vector2`].
    InvalidUnitDir,
}

// ------ TESTS
#[cfg(test)]
mod tests;
