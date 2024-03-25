//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

pub mod coords;
pub mod vector;
pub mod vertex;

// ------ IMPORTS

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

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
