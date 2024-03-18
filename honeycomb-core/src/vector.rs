//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{Coords2, CoordsFloat};

// ------ CONTENT

pub struct Vector2<T: CoordsFloat> {
    inner: Coords2<T>,
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
