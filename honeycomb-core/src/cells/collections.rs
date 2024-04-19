//! i-cell collection implementation
//!
//! This module contains all code used to model collection structures for i-cell identifiers. The
//! need for a specific structure stems from the need to ensure the validity of identifiers.

// ------ IMPORTS

use crate::{CMap2, CoordsFloat};

// ------ CONTENT

macro_rules! collection_constructor {
    ($coll: ident, $idty: ty) => {
        impl<'a, T: CoordsFloat> $coll<'a, T> {
            /// Constructor
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

/// Null value for vertex identifiers
pub const NULL_VERTEX_ID: VertexIdentifier = 0;

/// Vertex ID collection
///
/// # Generics
///
/// - `'a` -- Lifetime of a reference to the associated map.
/// - `T: CoordsFloat` -- Generic of the associated map.
///
/// # Example
///
/// See the [`CMap2`] quickstart example.
///
pub struct VertexCollection<'a, T: CoordsFloat> {
    /// Lifetime holder
    ///
    /// This is used to ensure that the collection is only used while valid, i.e. it is invalidated
    /// if the original map is used in a mutable context.
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    /// Collection of vertex identifiers.
    pub identifiers: Vec<VertexIdentifier>,
}

collection_constructor!(VertexCollection, VertexIdentifier);

// --- edges

/// Type definition for edge identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type EdgeIdentifier = u32;

/// Null value for edge identifiers
pub const NULL_EDGE_ID: EdgeIdentifier = 0;

/// Edge ID collection
///
/// # Generics
///
/// - `'a` -- Lifetime of a reference to the associated map.
/// - `T: CoordsFloat` -- Generic of the associated map.
///
/// # Example
///
/// See the [`CMap2`] quickstart example.
///
pub struct EdgeCollection<'a, T: CoordsFloat> {
    /// Lifetime holder
    ///
    /// This is used to ensure that the collection is only used while valid, i.e. it is invalidated
    /// if the original map is used in a mutable context.
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    /// Collection of vertex identifiers.
    pub identifiers: Vec<EdgeIdentifier>,
}

collection_constructor!(EdgeCollection, EdgeIdentifier);

// --- faces

/// Type definition for face identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type FaceIdentifier = u32;

/// Null value for face identifiers
pub const NULL_FACE_ID: FaceIdentifier = 0;

/// Face ID collection
///
/// # Generics
///
/// - `'a` -- Lifetime of a reference to the associated map.
/// - `T: CoordsFloat` -- Generic of the associated map.
///
/// # Example
///
/// See the [`CMap2`] quickstart example.
///
pub struct FaceCollection<'a, T: CoordsFloat> {
    /// Lifetime holder
    ///
    /// This is used to ensure that the collection is only used while valid, i.e. it is invalidated
    /// if the original map is used in a mutable context.
    lifetime_indicator: std::marker::PhantomData<&'a CMap2<T>>,
    /// Collection of vertex identifiers.
    pub identifiers: Vec<FaceIdentifier>,
}

collection_constructor!(FaceCollection, FaceIdentifier);

// --- volumes

/// Type definition for volume identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VolumeIdentifier = u32;

/// Null value for volume identifiers
pub const NULL_VOLUME_ID: VolumeIdentifier = 0;
