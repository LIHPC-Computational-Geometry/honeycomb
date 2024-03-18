//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::vector::Vector2;
use crate::{Coords2, CoordsFloat};

// ------ CONTENT

pub struct Vertex2<T: CoordsFloat> {
    inner: Coords2<T>,
}

// Building traits

impl<T: CoordsFloat> From<(T, T)> for Vertex2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self {
            inner: Coords2::from((x, y)),
        }
    }
}

impl<T: CoordsFloat> From<[T; 2]> for Vertex2<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self {
            inner: Coords2::from((x, y)),
        }
    }
}

// Basic operations

impl<T: CoordsFloat> std::ops::Add<Vector2<T>> for Vertex2<T> {
    // Vertex + Vector = Vertex
    type Output = Self;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        todo!()
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector2<T>> for Vertex2<T> {
    fn add_assign(&mut self, rhs: Vector2<T>) {
        todo!()
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vector2<T>> for Vertex2<T> {
    // Vertex - Vector = Vertex
    type Output = Self;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        todo!()
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector2<T>> for Vertex2<T> {
    fn sub_assign(&mut self, rhs: Vector2<T>) {
        todo!()
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
