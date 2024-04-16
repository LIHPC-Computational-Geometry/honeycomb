//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, EdgeIdentifier, FaceIdentifier, VertexIdentifier};

// ------ CONTENT

macro_rules! collection_constructor {
    ($coll: ident, $idty: ty) => {
        impl<'a, T: CoordsFloat> $coll<'a, T> {
            pub fn new(_: &'a CMap2<T>, ids: impl IntoIterator<Item = $idty>) -> Self {
                Self {
                    lifetime_indicator: std::marker::PhantomData::default(),
                    identifiers: ids.into_iter().collect(),
                }
            }
        }
    };
}

// --- vertices

pub struct VertexCollection<'a, T: CoordsFloat> {
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    pub identifiers: Vec<VertexIdentifier>,
}

collection_constructor!(VertexCollection, VertexIdentifier);

// --- edges

pub struct EdgeCollection<'a, T: CoordsFloat> {
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    pub identifiers: Vec<EdgeIdentifier>,
}

collection_constructor!(EdgeCollection, EdgeIdentifier);

// --- faces

pub struct FaceCollection<'a, T: CoordsFloat> {
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    pub identifiers: Vec<FaceIdentifier>,
}

collection_constructor!(FaceCollection, FaceIdentifier);
