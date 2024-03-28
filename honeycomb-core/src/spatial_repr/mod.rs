//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

// ------ CONTENT

use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

cfg_if::cfg_if! {
    if #[cfg(feature = "single_precision")] {
        pub type FloatType = f32;
    } else {
        pub type FloatType = f64;
    }
}

// ------ CONTENT

/// Common trait implemented by types used for coordinate representation.
pub trait CoordsFloat:
    num::Float + Default + AddAssign + SubAssign + MulAssign + DivAssign
{
}

impl CoordsFloat for f32 {}
impl CoordsFloat for f64 {}
