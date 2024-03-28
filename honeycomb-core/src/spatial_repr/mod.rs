//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

pub mod coords;
pub mod vector;
pub mod vertex;

// ------ RE-EXPORTS

pub use coords::{Coords2, CoordsError};
pub use vector::Vector2;
pub use vertex::Vertex2;

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
