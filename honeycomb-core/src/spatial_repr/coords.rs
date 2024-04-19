//! Custom spatial representation
//!
//! This module contains all code used to model spatial representation and
//! operations. We re-implement these basic structures in order to have
//! better control over our structure.

// ------ IMPORTS

#[cfg(doc)]
use crate::FloatType;
use crate::{CoordsFloat, Vector2, Vertex2};

use std::iter::Sum;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

// ------ CONTENT

/// Coordinates-related error enum
#[derive(Debug)]
pub enum CoordsError {
    /// Error during the computation of the unit directional vector.
    ///
    /// This is returned when trying to compute the unit vector of a null [Coords2] structure.
    InvalidUnitDir,
}

/// 2-dimensional coordinates structure
///
/// The floating type used for coordinate representation is determined by the user. For tests, it
/// can be controled using features and the [`FloatType`] alias.
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type for coordinates representation.
///
/// # Example
///
/// This type is not meant to be used directly when operating on combinatorial maps (see [Vector2],
/// [Vertex2] for that), but it is kept public because it is easier to use for rendering purposes.
/// As such, no example is provided.
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

impl<T: CoordsFloat> From<Vertex2<T>> for Coords2<T> {
    fn from(vector2: Vertex2<T>) -> Self {
        vector2.into_inner()
    }
}

impl<T: CoordsFloat> From<Vector2<T>> for Coords2<T> {
    fn from(vector2: Vector2<T>) -> Self {
        vector2.into_inner()
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

impl<T: CoordsFloat> Sum<Coords2<T>> for Coords2<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Coords2<T> {
        iter.fold(Self::default(), |c1, c2| c1 + c2)
    }
}

impl<'a, T: CoordsFloat> Sum<&'a Coords2<T>> for Coords2<T> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Coords2<T> {
        iter.fold(Self::default(), |c1, c2| c1 + *c2)
    }
}

impl<T: CoordsFloat> Index<usize> for Coords2<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            i => panic!("cannot index a 2D vector with value {i}"),
        }
    }
}

impl<T: CoordsFloat> IndexMut<usize> for Coords2<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            i => panic!("cannot index a 2D vector with value {i}"),
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
    fn sum() {
        let collection = [
            Coords2::unit_x(),
            Coords2::unit_x(),
            Coords2::unit_x(),
            Coords2::unit_y(),
            Coords2::unit_y(),
            Coords2::unit_y(),
        ];

        let owned_sum: Coords2<FloatType> = collection.into_iter().sum();
        let borrowed_sum: Coords2<FloatType> = collection.iter().sum();
        let ref_value: Coords2<FloatType> = Coords2::from((3.0, 3.0));
        assert!(almost_equal(&owned_sum, &ref_value));
        assert!(almost_equal(&borrowed_sum, &ref_value));
    }
}
