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

impl<T: CoordsFloat> Vertex2<T> {
    pub fn into_inner(self) -> Coords2<T> {
        self.inner
    }

    pub fn x(&self) -> T {
        self.inner.x
    }

    pub fn y(&self) -> T {
        self.inner.y
    }

    pub fn average(lhs: &Vertex2<T>, rhs: &Vertex2<T>) -> Vertex2<T> {
        let two = T::from(2.0).unwrap();
        Vertex2::from(((lhs.x() + rhs.x()) / two, (lhs.y() + rhs.y()) / two))
    }
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

impl<T: CoordsFloat> From<Coords2<T>> for Vertex2<T> {
    fn from(value: Coords2<T>) -> Self {
        Self { inner: value }
    }
}

// Basic operations

// -- add flavors

impl<T: CoordsFloat> std::ops::Add<Vector2<T>> for Vertex2<T> {
    // Vertex + Vector = Vertex
    type Output = Self;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Self {
            inner: self.inner + rhs.into_inner(),
        }
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector2<T>> for Vertex2<T> {
    fn add_assign(&mut self, rhs: Vector2<T>) {
        self.inner += rhs.into_inner()
    }
}

impl<T: CoordsFloat> std::ops::Add<&Vector2<T>> for Vertex2<T> {
    // Vertex + Vector = Vertex
    type Output = Self;

    fn add(self, rhs: &Vector2<T>) -> Self::Output {
        let mut tmp = self.inner;
        tmp.x += rhs.x();
        tmp.y += rhs.y();
        Self { inner: tmp }
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<&Vector2<T>> for Vertex2<T> {
    fn add_assign(&mut self, rhs: &Vector2<T>) {
        self.inner.x += rhs.x();
        self.inner.y += rhs.y();
    }
}

// -- sub flavors

impl<T: CoordsFloat> std::ops::Sub<Vector2<T>> for Vertex2<T> {
    // Vertex - Vector = Vertex
    type Output = Self;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Self {
            inner: self.inner - rhs.into_inner(),
        }
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector2<T>> for Vertex2<T> {
    fn sub_assign(&mut self, rhs: Vector2<T>) {
        self.inner -= rhs.into_inner();
    }
}

impl<T: CoordsFloat> std::ops::Sub<&Vector2<T>> for Vertex2<T> {
    // Vertex - Vector = Vertex
    type Output = Self;

    fn sub(self, rhs: &Vector2<T>) -> Self::Output {
        let mut tmp = self.inner;
        tmp.x -= rhs.x();
        tmp.y -= rhs.y();
        Self { inner: tmp }
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<&Vector2<T>> for Vertex2<T> {
    fn sub_assign(&mut self, rhs: &Vector2<T>) {
        self.inner.x -= rhs.x();
        self.inner.y -= rhs.y();
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vertex2<T>> for Vertex2<T> {
    type Output = Vector2<T>;

    fn sub(self, rhs: Vertex2<T>) -> Self::Output {
        Vector2::from(self.into_inner() - rhs.into_inner())
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
