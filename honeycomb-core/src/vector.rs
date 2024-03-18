//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{Coords2, CoordsError, CoordsFloat};

// ------ CONTENT

pub struct Vector2<T: CoordsFloat> {
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

    pub fn into_inner(self) -> Coords2<T> {
        self.inner
    }

    pub fn x(&self) -> T {
        self.inner.x
    }

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
    /// See [Coords2] example.
    ///
    pub fn norm(&self) -> T {
        self.inner.norm()
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
    /// See [Coords2] example.
    ///
    pub fn unit_dir(&self) -> Result<Vector2<T>, CoordsError> {
        self.inner
            .unit_dir()
            .map(|coords2: Coords2<T>| Self { inner: coords2 })
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
    /// See [Coords2] example.
    ///
    pub fn normal_dir(&self) -> Vector2<T> {
        Self {
            inner: self.inner.normal_dir(),
        }
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
    /// See [Coords2] example.
    ///
    pub fn dot(&self, other: &Vector2<T>) -> T {
        self.inner.dot(&other.inner)
    }
}

// Building traits

impl<T: CoordsFloat> From<(T, T)> for Vector2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self {
            inner: Coords2::from((x, y)),
        }
    }
}

impl<T: CoordsFloat> From<[T; 2]> for Vector2<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self {
            inner: Coords2::from((x, y)),
        }
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
