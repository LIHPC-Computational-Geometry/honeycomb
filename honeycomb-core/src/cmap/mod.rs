//! combinatorial maps implementations

mod builder;
mod components;
mod dim2;
mod error;

pub use builder::{BuilderError, CMapBuilder};
pub use components::{
    collections::{EdgeCollection, FaceCollection, VertexCollection},
    identifiers::{
        DartIdType, EdgeIdType, FaceIdType, VertexIdType, VolumeIdType, NULL_DART_ID, NULL_EDGE_ID,
        NULL_FACE_ID, NULL_VERTEX_ID, NULL_VOLUME_ID,
    },
    orbits::OrbitPolicy,
};
pub use dim2::{orbits::Orbit2, structure::CMap2};
pub use error::{CMapError, CMapResult};

#[cfg(feature = "utils")]
pub use builder::GridDescriptor;
