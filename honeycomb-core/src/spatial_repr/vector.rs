//! Custom spatial representation
//!
//! This module contains all code used to model vectors as wrappers of a common
//! type ([Coords2]).

// ------ IMPORTS

use crate::{CoordsError, CoordsFloat};

// ------ CONTENT

/// 2D vector representation
///
/// This structure is a wrapper around a [Coords2] value. Defining this as a wrapper
/// instead of a simple type alias allow us to introduce the notion of homogeneity.
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type for coordinates representation.
///
/// # Example
///
/// ```
/// # use honeycomb_core::CoordsError;
/// # fn main() -> Result<(), CoordsError> {
/// use honeycomb_core::Vector2;
///
/// let unit_x = Vector2::unit_x();
/// let unit_y = Vector2::unit_y();
///
/// assert_eq!(unit_x.dot(&unit_y), 0.0);
/// assert_eq!(unit_x.normal_dir().unwrap(), unit_y);
///
/// let two: f64 = 2.0;
/// let x_plus_y: Vector2<f64> = unit_x + unit_y;
///
/// assert_eq!(x_plus_y.norm(), two.sqrt());
/// assert_eq!(x_plus_y.unit_dir()?, Vector2(1.0 / two.sqrt(), 1.0 / two.sqrt()));
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2<T: CoordsFloat>(pub T, pub T);

impl<T: CoordsFloat> Vector2<T> {
    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `x` axis.
    ///
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn unit_x() -> Self {
        Self(T::one(), T::zero())
    }

    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `y` axis.
    ///
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn unit_y() -> Self {
        Self(T::zero(), T::one())
    }

    /// Consume `self` to return inner value
    ///
    /// # Return
    ///
    /// Return a [Coords2] object.
    ///
    pub fn into_inner(self) -> (T, T) {
        (self.0, self.1)
    }

    /// Getter
    ///
    /// # Return
    ///
    /// Return the value of the `x` coordinate of the vector.
    ///
    pub fn x(&self) -> T {
        self.0
    }

    /// Getter
    ///
    /// # Return
    ///
    /// Return the value of the `y` coordinate of the vector.
    ///
    pub fn y(&self) -> T {
        self.1
    }

    /// Compute the norm of `self`.
    ///
    /// # Return
    ///
    /// Return the norm. Its type is the same as the one used for internal
    /// representation.
    ///
    /// # Example
    ///
    /// See [Vector2] example.
    ///
    pub fn norm(&self) -> T {
        self.0.hypot(self.1)
    }

    /// Compute the direction of `self` as a unit vector.
    ///
    /// # Return
    ///
    /// Return a [Vector2] indicating the direction of `self`. The norm of the returned
    /// struct is equal to one.
    ///
    /// # Errors
    ///
    /// This method will return an error if called on a `Vector2` with a norm equal to zero,
    /// i.e. a null `Vector2`.
    ///
    /// # Example
    ///
    /// See [Vector2] example.
    ///
    pub fn unit_dir(&self) -> Result<Self, CoordsError> {
        let norm = self.norm();
        if norm.is_zero() {
            Err(CoordsError::InvalidUnitDir)
        } else {
            Ok(*self / norm)
        }
    }

    /// Compute the direction of the normal vector to `self`.
    ///
    /// # Return
    ///
    /// Return a [Vector2] indicating the direction of the normal to `self`. The norm of the
    /// returned struct is equal to one.
    ///
    /// # Errors
    ///
    /// This method will return an error if called on a `Vector2` with a norm equal to zero,
    /// i.e. a null `Vector2`.
    ///
    /// # Example
    ///
    /// See [Vector2] example.
    ///
    pub fn normal_dir(&self) -> Result<Vector2<T>, CoordsError> {
        Self(-self.1, self.0).unit_dir() // unit(-y, x)
    }

    /// Compute the dot product between two vectors
    ///
    /// # Arguments
    ///
    /// - `other: &Vector2` -- reference to the second vector.
    ///
    /// # Return
    ///
    /// Return the dot product between `self` and `other`.
    ///
    /// # Example
    ///
    /// See [Vector2] example.
    ///
    pub fn dot(&self, other: &Vector2<T>) -> T {
        self.0 * other.0 + self.1 * other.1
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
