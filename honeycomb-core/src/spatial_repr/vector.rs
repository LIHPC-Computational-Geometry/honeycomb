//! Custom spatial representation
//!
//! This module contains all code used to model vectors as wrappers of a common
//! type ([Coords2]).
//!

// ------ IMPORTS

use crate::{Coords2, CoordsError, CoordsFloat};

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
/// use honeycomb_core::{Vector2, FloatType};
///
/// let unit_x = Vector2::unit_x();
/// let unit_y = Vector2::unit_y();
///
/// assert_eq!(unit_x.dot(&unit_y), 0.0);
/// assert_eq!(unit_x.normal_dir(), unit_y);
///
/// let two: FloatType = 2.0;
/// let x_plus_y: Vector2<FloatType> = unit_x + unit_y;
///
/// assert_eq!(x_plus_y.norm(), two.sqrt());
/// assert_eq!(x_plus_y.unit_dir()?, Vector2::from((1.0 / two.sqrt(), 1.0 / two.sqrt())));
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2<T: CoordsFloat> {
    /// Coordinates value.
    inner: Coords2<T>,
}

impl<T: CoordsFloat> Vector2<T> {
    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `x` axis.
    ///
    pub fn unit_x() -> Self {
        Self {
            inner: Coords2::unit_x(),
        }
    }

    /// Base vector
    ///
    /// # Return
    ///
    /// Return a unit vector along the `y` axis.
    ///
    pub fn unit_y() -> Self {
        Self {
            inner: Coords2::unit_y(),
        }
    }

    /// Consume `self` to return inner value
    ///
    /// # Return
    ///
    /// Return a [Coords2] object.
    ///
    pub fn into_inner(self) -> Coords2<T> {
        self.inner
    }

    /// Getter
    ///
    /// # Return
    ///
    /// Return the value of the `x` coordinate of the vector.
    ///
    pub fn x(&self) -> T {
        self.inner.x
    }

    /// Getter
    ///
    /// # Return
    ///
    /// Return the value of the `y` coordinate of the vector.
    ///
    pub fn y(&self) -> T {
        self.inner.y
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
        self.inner.x.hypot(self.inner.y)
    }

    /// Compute the direction of `self` as a unit vector.
    ///
    /// # Return
    ///
    /// Return a [Vector2] indicating the direction of `self`. The norm of the returned
    /// struct is equal to one.
    ///
    /// # Example
    ///
    /// See [Vector2] example.
    ///
    pub fn unit_dir(&self) -> Result<Vector2<T>, CoordsError> {
        let norm = self.norm();
        if !norm.is_zero() {
            Ok(*self / norm)
        } else {
            Err(CoordsError::InvalidUnitDir)
        }
    }

    /// Compute the direction of the normal vector to `self`.
    ///
    /// # Return
    ///
    /// Return a [Vector2] indicating the direction of the normal to `self`. The norm of the
    /// returned struct is equal to one.
    ///
    /// # Example
    ///
    /// See [Vector2] example.
    ///
    pub fn normal_dir(&self) -> Vector2<T> {
        Self {
            inner: Coords2 {
                x: -self.inner.y,
                y: self.inner.x,
            },
        }
        .unit_dir()
        .unwrap()
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
        self.inner.x * other.inner.x + self.inner.y * other.inner.y
    }
}

// Building traits

macro_rules! impl_from_for_vector {
    ($src_type: ty) => {
        impl<T: CoordsFloat> From<$src_type> for Vector2<T> {
            fn from(value: $src_type) -> Self {
                Self {
                    inner: Coords2::from(value),
                }
            }
        }
    };
}

impl_from_for_vector!((T, T));
impl_from_for_vector!([T; 2]);
impl_from_for_vector!(Coords2<T>);

// Basic operations

impl<T: CoordsFloat> std::ops::Add<Vector2<T>> for Vector2<T> {
    type Output = Self;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Self {
            inner: self.inner + rhs.into_inner(),
        }
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector2<T>> for Vector2<T> {
    fn add_assign(&mut self, rhs: Vector2<T>) {
        self.inner += rhs.into_inner();
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vector2<T>> for Vector2<T> {
    type Output = Self;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Self {
            inner: self.inner - rhs.into_inner(),
        }
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector2<T>> for Vector2<T> {
    fn sub_assign(&mut self, rhs: Vector2<T>) {
        self.inner -= rhs.into_inner()
    }
}

impl<T: CoordsFloat> std::ops::Mul<T> for Vector2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            inner: self.inner * rhs,
        }
    }
}

impl<T: CoordsFloat> std::ops::MulAssign<T> for Vector2<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.inner *= rhs;
    }
}

impl<T: CoordsFloat> std::ops::Div<T> for Vector2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        // there is an assert in the Coords2 impl but
        // putting one here will shorten the stack trace
        assert!(!rhs.is_zero());
        Self {
            inner: self.inner / rhs,
        }
    }
}

impl<T: CoordsFloat> std::ops::DivAssign<T> for Vector2<T> {
    fn div_assign(&mut self, rhs: T) {
        // there is an assert in the Coords2 impl but
        // putting one here will shorten the stack trace
        assert!(!rhs.is_zero());
        self.inner /= rhs;
    }
}

impl<T: CoordsFloat> std::ops::Neg for Vector2<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { inner: -self.inner }
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::*;

    use crate::FloatType;

    fn almost_equal(lhs: &Vector2<FloatType>, rhs: &Vector2<FloatType>) -> bool {
        const EPS: FloatType = 10.0e-12;
        ((lhs.x() - rhs.x()).abs() < EPS) & ((lhs.y() - rhs.y()).abs() < EPS)
    }

    #[test]
    fn dot_product() {
        let along_x = Vector2::unit_x() * 15.0;
        let along_y = Vector2::unit_y() * 10.0;
        assert_eq!(along_x.dot(&along_y), 0.0);
        assert_eq!(along_x.dot(&Vector2::unit_x()), 15.0);
        assert_eq!(along_y.dot(&Vector2::unit_y()), 10.0);
    }

    #[test]
    fn unit_dir() {
        let along_x = Vector2::unit_x() * 4.0;
        let along_y = Vector2::unit_y() * 3.0;
        assert_eq!(along_x.unit_dir().unwrap(), Vector2::unit_x());
        assert_eq!(
            Vector2::<FloatType>::unit_x().unit_dir().unwrap(),
            Vector2::unit_x()
        );
        assert_eq!(along_y.unit_dir().unwrap(), Vector2::unit_y());
        assert!(almost_equal(
            &(along_x + along_y).unit_dir().unwrap(),
            &Vector2::from((4.0 / 5.0, 3.0 / 5.0))
        ));
        let origin: Vector2<FloatType> = Vector2::default();
        assert!(origin.unit_dir().is_err());
    }
}
