//! i-cell collection implementation
//!
//! This module contains all code used to model collection structures for i-cell identifiers. The
//! need for a specific structure stems from the need to ensure the validity of identifiers.

// ------ IMPORTS

use crate::cmap::{EdgeIdType, FaceIdType, VertexIdType};
use crate::geometry::CoordsFloat;
use crate::prelude::CMap2;

// ------ CONTENT

macro_rules! collection_constructor {
    ($coll: ident, $idty: ty) => {
        impl<'a, T: CoordsFloat> $coll<'a, T> {
            /// Constructor
            pub(crate) fn new(_: &'a CMap2<T>, ids: impl IntoIterator<Item = $idty>) -> Self {
                Self {
                    lifetime_indicator: std::marker::PhantomData::default(),
                    identifiers: ids.into_iter().collect(),
                }
            }
        }
    };
}

// --- vertices

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
    pub identifiers: Vec<VertexIdType>,
}

unsafe impl<'a, T: CoordsFloat> Send for VertexCollection<'a, T> {}
unsafe impl<'a, T: CoordsFloat> Sync for VertexCollection<'a, T> {}

collection_constructor!(VertexCollection, VertexIdType);

// --- edges

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
    pub identifiers: Vec<EdgeIdType>,
}

unsafe impl<'a, T: CoordsFloat> Send for EdgeCollection<'a, T> {}
unsafe impl<'a, T: CoordsFloat> Sync for EdgeCollection<'a, T> {}

collection_constructor!(EdgeCollection, EdgeIdType);

// --- faces

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
    pub identifiers: Vec<FaceIdType>,
}

unsafe impl<'a, T: CoordsFloat> Send for FaceCollection<'a, T> {}
unsafe impl<'a, T: CoordsFloat> Sync for FaceCollection<'a, T> {}

collection_constructor!(FaceCollection, FaceIdType);

// --- volumes
