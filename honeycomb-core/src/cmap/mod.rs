//! combinatorial maps implementations

mod builder;
mod components;
mod dim2;
#[allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)] // FIXME:write docs
mod dim3;
mod error;

pub use builder::{BuilderError, CMapBuilder, GridDescriptor};
pub use components::{
    identifiers::{
        DartIdType, EdgeIdType, FaceIdType, NULL_DART_ID, NULL_EDGE_ID, NULL_FACE_ID,
        NULL_VERTEX_ID, NULL_VOLUME_ID, VertexIdType, VolumeIdType,
    },
    orbits::OrbitPolicy,
};
pub use dim2::structure::CMap2;
pub use dim3::structure::CMap3;
pub use error::{DartReleaseError, DartReservationError, LinkError, SewError};

pub(crate) use components::orbits::try_from_fn;
