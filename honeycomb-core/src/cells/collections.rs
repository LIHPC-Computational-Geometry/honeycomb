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

macro_rules! collection_constructor {
    ($coll: ident, $idty: ty) => {
        impl<'a, T: CoordsFloat> $coll<'a, T> {
            pub fn new(map_handle: &'a CMap2<T>, ids: impl IntoIterator<Item = $idty>) -> Self {
                Self {
                    map: map_handle,
                    identifiers: ids.into_iter().collect(),
                }
            }
        }
    };
}

collection_constructor!(VertexCollection, VertexIdentifier);
collection_constructor!(EdgeCollection, EdgeIdentifier);
collection_constructor!(FaceCollection, FaceIdentifier);

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
