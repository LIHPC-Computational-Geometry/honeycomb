//! combinatorial maps implementations

mod builder;
mod collections;
mod components;
mod dim2;

pub use builder::{BuilderError, CMapBuilder};
pub use collections::{EdgeCollection, FaceCollection, VertexCollection};
pub use components::identifiers::{
    CellId, DartId, DartIdType, EdgeId, EdgeIdType, FaceId, FaceIdType, VertexId, VertexIdType,
    VolumeId, VolumeIdType, NULL_DART_ID, NULL_EDGE_ID, NULL_FACE_ID, NULL_VERTEX_ID,
    NULL_VOLUME_ID,
};
pub use dim2::{
    orbits::{Orbit2, OrbitPolicy},
    structure::CMap2,
};

#[cfg(feature = "utils")]
pub use builder::GridDescriptor;
