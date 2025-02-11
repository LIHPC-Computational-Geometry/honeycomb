//! Custom spatial representation
//!
//! This module contains all code used to model vectors.

use crate::geometry::{CoordsError, CoordsFloat};

/// # 2D vector structure
///
/// ## Generics
///
/// - `T: CoordsFloat` -- Generic FP type for coordinates.
///
/// ## Example
///
/// ```
/// # use honeycomb_core::prelude::CoordsError;
/// # fn main() -> Result<(), CoordsError> {
/// use honeycomb_core::prelude::Vector2;
///
/// let unit_x = Vector2::unit_x();
/// let unit_y = Vector2::unit_y();
///
/// assert_eq!(unit_x.dot(&unit_y), 0.0);
/// assert_eq!(unit_x.normal_dir()?, unit_y);
///
/// let two: f64 = 2.0;
/// let x_plus_y: Vector2<f64> = unit_x + unit_y;
///
/// assert_eq!(x_plus_y.norm(), two.sqrt());
/// assert_eq!(x_plus_y.unit_dir()?, Vector2(1.0 / two.sqrt(), 1.0 / two.sqrt()));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2<T: CoordsFloat>(pub T, pub T);

unsafe impl<T: CoordsFloat> Send for Vector2<T> {}
unsafe impl<T: CoordsFloat> Sync for Vector2<T> {}

impl<T: CoordsFloat> Vector2<T> {
    /// Return a unit vector along the `x` axis.
    #[must_use = "unused return value"]
    pub fn unit_x() -> Self {
        Self(T::one(), T::zero())
    }

    /// Return a unit vector along the `y` axis.
    #[must_use = "unused return value"]
    pub fn unit_y() -> Self {
        Self(T::zero(), T::one())
    }

    /// Consume `self` to return inner values.
    pub fn into_inner(self) -> (T, T) {
        (self.0, self.1)
    }

    /// Return the value of the `x` coordinate of the vector.
    pub fn x(&self) -> T {
        self.0
    }

    /// Return the value of the `y` coordinate of the vector.
    pub fn y(&self) -> T {
        self.1
    }

    /// Compute the norm of `self`.
    pub fn norm(&self) -> T {
        self.0.hypot(self.1)
    }

    /// Compute the direction of `self` as a unit vector.
    ///
    /// # Errors
    ///
    /// This method will return an error if called on a null `Vector2`.
    pub fn unit_dir(&self) -> Result<Self, CoordsError> {
        let norm = self.norm();
        if norm.is_zero() {
            Err(CoordsError::InvalidUnitDir)
        } else {
            Ok(*self / norm)
        }
    }

    /// Compute the direction of the normal vector to `self` as a unit vector.
    ///
    /// # Errors
    ///
    /// This method will return an error if called on a null `Vector2`.
    pub fn normal_dir(&self) -> Result<Vector2<T>, CoordsError> {
        Self(-self.1, self.0)
            .unit_dir() // unit(-y, x)
            .map_err(|_| CoordsError::InvalidNormDir)
    }

    /// Return the dot product between `self` and `other`.
    pub fn dot(&self, other: &Vector2<T>) -> T {
        self.0 * other.0 + self.1 * other.1
    }
}

// Building trait

impl<T: CoordsFloat> From<(T, T)> for Vector2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self(x, y)
    }
}

// Basic operations

impl<T: CoordsFloat> std::ops::Add<Vector2<T>> for Vector2<T> {
    type Output = Self;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector2<T>> for Vector2<T> {
    fn add_assign(&mut self, rhs: Vector2<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vector2<T>> for Vector2<T> {
    type Output = Self;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector2<T>> for Vector2<T> {
    fn sub_assign(&mut self, rhs: Vector2<T>) {
        self.0 -= rhs.0;
        self.0 -= rhs.0;
    }
}

impl<T: CoordsFloat> std::ops::Mul<T> for Vector2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl<T: CoordsFloat> std::ops::MulAssign<T> for Vector2<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
        self.1 *= rhs;
    }
}

impl<T: CoordsFloat> std::ops::Div<T> for Vector2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        assert!(!rhs.is_zero());
        Self(self.0 / rhs, self.1 / rhs)
    }
}

impl<T: CoordsFloat> std::ops::DivAssign<T> for Vector2<T> {
    fn div_assign(&mut self, rhs: T) {
        assert!(!rhs.is_zero());
        self.0 /= rhs;
        self.1 /= rhs;
    }
}

impl<T: CoordsFloat> std::ops::Neg for Vector2<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}
