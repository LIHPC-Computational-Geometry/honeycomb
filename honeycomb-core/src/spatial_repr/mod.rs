//! Spatial representation types & operators
//!
//! This module contains all code related to custom spatial reprentation type implementations.
//! This include custom vector / vertex types as well as generic traits for value encoding.

// ------ MODULE DECLARATIONS

pub mod coords;
pub mod vector;
pub mod vertex;

// ------ IMPORTS

use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

// ------ CONTENT

cfg_if::cfg_if! {
    if #[cfg(feature = "single_precision")] {
        pub type FloatType = f32;
    } else {
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
