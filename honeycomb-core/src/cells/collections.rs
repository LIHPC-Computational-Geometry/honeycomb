//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CMap2, CoordsFloat};

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

/// Type definition for vertex identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VertexIdentifier = u32;

pub const NULL_VERTEX_ID: VertexIdentifier = 0;

pub struct VertexCollection<'a, T: CoordsFloat> {
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    pub identifiers: Vec<VertexIdentifier>,
}

collection_constructor!(VertexCollection, VertexIdentifier);

// --- edges

/// Type definition for edge identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type EdgeIdentifier = u32;

pub const NULL_EDGE_ID: EdgeIdentifier = 0;

pub struct EdgeCollection<'a, T: CoordsFloat> {
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    pub identifiers: Vec<EdgeIdentifier>,
}

collection_constructor!(EdgeCollection, EdgeIdentifier);

// --- faces

/// Type definition for face identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type FaceIdentifier = u32;

pub const NULL_FACE_ID: FaceIdentifier = 0;

pub struct FaceCollection<'a, T: CoordsFloat> {
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    pub identifiers: Vec<FaceIdentifier>,
}

collection_constructor!(FaceCollection, FaceIdentifier);

// --- volumes

/// Type definition for volume identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VolumeIdentifier = u32;

pub const NULL_VOLUME_ID: VolumeIdentifier = 0;
