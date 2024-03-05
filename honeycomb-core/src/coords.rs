//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

use std::ops::{Add, AddAssign, Sub, SubAssign};

// ------ CONTENT

cfg_if::cfg_if! {
    if #[cfg(feature = "single_precision")] {
        pub type FloatType = f32;
    } else {
        pub type FloatType = f64;
    }
}

pub struct Coords2 {
    pub x: FloatType,
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

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
