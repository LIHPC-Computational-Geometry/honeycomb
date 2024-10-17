// ------ COMMON RE-EXPORTS

pub use crate::attributes::{AttributeBind, AttributeUpdate};
pub use crate::cmap::{
    CMap2, EdgeIdentifier, FaceIdentifier, Orbit2, OrbitPolicy, VertexIdentifier, VolumeIdentifier,
    NULL_EDGE_ID, NULL_FACE_ID, NULL_VERTEX_ID, NULL_VOLUME_ID,
};
pub use crate::common::{BuilderError, CMapBuilder, DartIdentifier, NULL_DART_ID};
pub use crate::geometry::{CoordsError, CoordsFloat, Vector2, Vertex2};

// ------ FEATURE-GATED RE-EXPORTS

#[cfg(feature = "utils")]
pub use crate::common::GridDescriptor;
