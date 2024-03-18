//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{Coords2, CoordsFloat};

// ------ CONTENT

pub struct Vertex2<T: CoordsFloat> {
    inner: Coords2<T>,
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
