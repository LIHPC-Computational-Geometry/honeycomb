//! combinatorial maps implementations

mod builder;
mod collections;
mod common;
#[allow(unused)]
mod components;
mod dim2;

pub use builder::{BuilderError, CMapBuilder};
pub use collections::{
    EdgeCollection, EdgeIdentifier, FaceCollection, FaceIdentifier, VertexCollection,
    VertexIdentifier, VolumeIdentifier, NULL_EDGE_ID, NULL_FACE_ID, NULL_VERTEX_ID, NULL_VOLUME_ID,
};
pub use common::{DartIdentifier, NULL_DART_ID};
pub use dim2::{
    orbits::{Orbit2, OrbitPolicy},
    structure::CMap2,
};

#[cfg(feature = "utils")]
pub use builder::GridDescriptor;
