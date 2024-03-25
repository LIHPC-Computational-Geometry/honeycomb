//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, EdgeIdentifier, FaceIdentifier, VertexIdentifier};

// ------ CONTENT

pub struct VertexCollection<'a, T: CoordsFloat> {
    map: &'a CMap2<T>,
    pub identifiers: Vec<VertexIdentifier>,
}

pub struct EdgeCollection<'a, T: CoordsFloat> {
    map: &'a CMap2<T>,
    pub identifiers: Vec<EdgeIdentifier>,
}

pub struct FaceCollection<'a, T: CoordsFloat> {
    map: &'a CMap2<T>,
    pub identifiers: Vec<FaceIdentifier>,
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
