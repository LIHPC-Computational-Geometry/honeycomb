//! Custom spatial representation
//!
//! This module contains all code used to model spatial representation and
//! operations. We re-implement these basic structures in order to have
//! better control over our structure.

// ------ IMPORTS

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// ------ CONTENT

pub trait CoordsFloat:
    num::Float + Default + AddAssign + SubAssign + MulAssign + DivAssign
{
}

impl CoordsFloat for f32 {}
impl CoordsFloat for f64 {}

/// 2-dimensional coordinates structure
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type for coordinates representation.
///
/// # Example
///
/// ```text
/// use honeycomb_core::{Coords2, FloatType};
///
/// let unit_x = Coords2::unit_x();
/// let unit_y = Coords2::unit_y();
///
/// assert_eq!(unit_x.dot(&unit_y), 0.0);
/// assert_eq!(unit_x.normal_dir(), unit_y);
///
/// let two: FloatType = 2.0;
/// let x_plus_y: Coords2 = unit_x + unit_y;
///
/// assert_eq!(x_plus_y.norm(), two.sqrt());
/// assert_eq!(x_plus_y.unit_dir(), Coords2::from((1.0 / two.sqrt(), 1.0 / two.sqrt())));
/// ```
///
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Coords2<T: CoordsFloat> {
    /// First coordinate
    pub x: T,
    /// Second coordinate
    pub y: T,
}

impl<T: CoordsFloat> Coords2<T> {
    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `x` axis.
    ///
    pub fn unit_x() -> Coords2<T> {
        Self {
            x: T::one(),
            y: T::zero(),
        }
    }

    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `y` axis.
    ///
    pub fn unit_y() -> Coords2<T> {
        Self {
            x: T::zero(),
            y: T::one(),
        }
    }

    /// Computes the norm of `self`.
    ///
    /// # Return
    ///
    /// Return the norm. Its type is the same as the one used for internal
    /// representation.
    ///
    /// # Example
    ///
    /// See [Coords2] example.
    ///
    pub fn norm(&self) -> T {
        self.x.hypot(self.y)
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
    pub fn unit_dir(&self) -> Coords2<T> {
        *self / self.norm()
    }

    /// Computes the direction of the normal vector to `self`.
    ///
    /// # Return
    ///
    /// Return a [Coords2] indicating the direction of the normal to `self`. The norm of the
    /// returned struct is equal to one.
    ///
    /// # Example
    ///
    /// See [Coords2] example.
    ///
    pub fn normal_dir(&self) -> Coords2<T> {
        Coords2 {
            x: -self.y,
            y: self.x,
        }
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
    pub fn dot(&self, other: &Coords2<T>) -> T {
        self.x * other.x + self.y * other.y
    }
}

// Building traits

impl<T: CoordsFloat> From<(T, T)> for Coords2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl<T: CoordsFloat> From<[T; 2]> for Coords2<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self { x, y }
    }
}

// Basic operations

impl<T: CoordsFloat> Add<Coords2<T>> for Coords2<T> {
    type Output = Self;

    fn add(self, rhs: Coords2<T>) -> Self::Output {
        Self::from((self.x + rhs.x, self.y + rhs.y))
    }
}

impl<T: CoordsFloat> AddAssign<Coords2<T>> for Coords2<T> {
    fn add_assign(&mut self, rhs: Coords2<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: CoordsFloat> Sub<Coords2<T>> for Coords2<T> {
    type Output = Self;

    fn sub(self, rhs: Coords2<T>) -> Self::Output {
        Self::from((self.x - rhs.x, self.y - rhs.y))
    }
}

impl<T: CoordsFloat> SubAssign<Coords2<T>> for Coords2<T> {
    fn sub_assign(&mut self, rhs: Coords2<T>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: CoordsFloat> Mul<T> for Coords2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::from((self.x * rhs, self.y * rhs))
    }
}

impl<T: CoordsFloat> MulAssign<T> for Coords2<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: CoordsFloat> Div<T> for Coords2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        assert!(!rhs.is_zero());
        Coords2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: CoordsFloat> DivAssign<T> for Coords2<T> {
    fn div_assign(&mut self, rhs: T) {
        assert!(!rhs.is_zero());
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<T: CoordsFloat> Neg for Coords2<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FloatType;

    fn almost_equal(lhs: &Coords2<FloatType>, rhs: &Coords2<FloatType>) -> bool {
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
        assert_eq!(Coords2::<FloatType>::unit_x().unit_dir(), Coords2::unit_x());
        assert_eq!(along_y.unit_dir(), Coords2::unit_y());
        assert!(almost_equal(
            &(along_x + along_y).unit_dir(),
            &Coords2::from((4.0 / 5.0, 3.0 / 5.0))
        ));
    }
}
