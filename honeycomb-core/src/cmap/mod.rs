//! combinatorial maps implementations

mod builder;
mod components;
mod dim2; // FIXME:simplify docs
#[allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)] // FIXME:write docs
mod dim3;
mod error;

pub use builder::{BuilderError, CMapBuilder, GridDescriptor};
pub use components::{
    identifiers::{
        DartIdType, EdgeIdType, FaceIdType, VertexIdType, VolumeIdType, NULL_DART_ID, NULL_EDGE_ID,
        NULL_FACE_ID, NULL_VERTEX_ID, NULL_VOLUME_ID,
    },
    orbits::OrbitPolicy,
};
pub use dim2::{orbits::Orbit2, structure::CMap2};
pub use dim3::{orbits::Orbit3, structure::CMap3};
pub use error::{LinkError, SewError};
