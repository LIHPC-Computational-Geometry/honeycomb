//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// ------ CONTENT

cfg_if::cfg_if! {
    if #[cfg(feature = "single_precision")] {
        pub type FloatType = f32;
    } else {
        pub type FloatType = f64;
    }
}

/// 2-dimensional coordinates structure
///
/// The floating type used for coordinate representation is determined
/// using feature and the [FloatType] alias.
///
/// # Example
///
/// ```text
///
/// ```
///
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Coords2 {
    /// First coordinate
    pub x: FloatType,
    /// Second coordinate
    pub y: FloatType,
}

// Building traits

impl From<(FloatType, FloatType)> for Coords2 {
    fn from((x, y): (FloatType, FloatType)) -> Self {
        Self { x, y }
    }
}

impl From<[FloatType; 2]> for Coords2 {
    fn from([x, y]: [FloatType; 2]) -> Self {
        Self { x, y }
    }
}

// Basic operations

impl Add<Coords2> for Coords2 {
    type Output = Self;

    fn add(self, rhs: Coords2) -> Self::Output {
        Self::from((self.x + rhs.x, self.y + rhs.y))
    }
}

impl AddAssign<Coords2> for Coords2 {
    fn add_assign(&mut self, rhs: Coords2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Coords2> for Coords2 {
    type Output = Self;

    fn sub(self, rhs: Coords2) -> Self::Output {
        Self::from((self.x - rhs.x, self.y - rhs.y))
    }
}

impl SubAssign<Coords2> for Coords2 {
    fn sub_assign(&mut self, rhs: Coords2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<FloatType> for Coords2 {
    type Output = Self;

    fn mul(self, rhs: FloatType) -> Self::Output {
        Self::from((self.x * rhs, self.y * rhs))
    }
}

impl MulAssign<FloatType> for Coords2 {
    fn mul_assign(&mut self, rhs: FloatType) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<FloatType> for Coords2 {
    type Output = Self;

    fn div(self, rhs: FloatType) -> Self::Output {
        assert_ne!(rhs, 0.0);
        self * 1.0 / rhs
    }
}

impl DivAssign<FloatType> for Coords2 {
    fn div_assign(&mut self, rhs: FloatType) {
        assert_ne!(rhs, 0.0);
        *self *= 1.0 / rhs;
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
