//! Custom spatial representation
//!
//! This module contains all code used to model spatial representation and
//! operations. We re-implement these basic structures in order to have
//! better control over our structure.

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
/// ```rust
/// use honeycomb_core::{Coords2, FloatType};
///
/// let unit_x = Coords2::unit_x();
/// let unit_y = Coords2::unit_y();
///
/// assert_eq!(unit_x.dot(&unit_y), 0.0);
///
/// let two: FloatType = 2.0;
/// let x_plus_y: Coords2 = unit_x + unit_y;
///
/// assert_eq!(x_plus_y.norm(), two.sqrt());
/// assert_eq!(x_plus_y.unit_dir(), Coords2::from((1.0 / two.sqrt(), 1.0 / two.sqrt())));
/// ```
///
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Coords2 {
    /// First coordinate
    pub x: FloatType,
    /// Second coordinate
    pub y: FloatType,
}

impl Coords2 {
    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `x` axis.
    ///
    pub fn unit_x() -> Coords2 {
        Self { x: 1., y: 0. } // make it a const?
    }

    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `y` axis.
    ///
    pub fn unit_y() -> Coords2 {
        Self { x: 0., y: 1. } // make it a const?
    }

    /// Computes the norm of `self`.
    ///
    /// # Return
    ///
    /// Return the norm as a [FloatType].
    ///
    /// # Example
    ///
    /// See [Coords2] example.
    ///
    pub fn norm(&self) -> FloatType {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Computes the direction of `self` as a unit vector.
    ///
    /// # Return
    ///
    /// Return a [Coords2] indicating the direction of `self`. The norm of the returned
    /// struct is equal to one.
    ///
    /// # Example
    ///
    /// See [Coords2] example.
    ///
    pub fn unit_dir(&self) -> Coords2 {
        *self / self.norm()
    }

    /// Computes the dot product between two vectors
    ///
    /// # Arguments
    ///
    /// - `other: &Coords2` -- reference to the second vector.
    ///
    /// # Return
    ///
    /// Return the dot product between `self` and `other`.
    ///
    /// # Example
    ///
    /// See [Coords2] example.
    ///
    pub fn dot(&self, other: &Coords2) -> FloatType {
        self.x * other.x + self.y * other.y
    }
}

// Building traits

impl<T: Into<FloatType>> From<(T, T)> for Coords2 {
    fn from((x, y): (T, T)) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<T: Into<FloatType>> From<[T; 2]> for Coords2 {
    fn from([x, y]: [T; 2]) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
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
        self * (1.0 / rhs)
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
    use super::*;

    fn almost_equal(lhs: &Coords2, rhs: &Coords2) -> bool {
        const EPS: FloatType = 10.0e-12;
        ((lhs.x - rhs.x).abs() < EPS) & ((lhs.y - rhs.y).abs() < EPS)
    }

    #[test]
    fn dot_product() {
        let along_x = Coords2::unit_x() * 15.0;
        let along_y = Coords2::unit_y() * 10.0;
        assert_eq!(along_x.dot(&along_y), 0.0);
        assert_eq!(along_x.dot(&Coords2::unit_x()), 15.0);
        assert_eq!(along_y.dot(&Coords2::unit_y()), 10.0);
    }

    #[test]
    fn unit_dir() {
        let along_x = Coords2::unit_x() * 4.0;
        let along_y = Coords2::unit_y() * 3.0;
        assert_eq!(along_x.unit_dir(), Coords2::unit_x());
        assert_eq!(Coords2::unit_x().unit_dir(), Coords2::unit_x());
        assert_eq!(along_y.unit_dir(), Coords2::unit_y());
        assert!(almost_equal(
            &(along_x + along_y).unit_dir(),
            &Coords2::from((4.0 / 5.0, 3.0 / 5.0))
        ));
    }
}
