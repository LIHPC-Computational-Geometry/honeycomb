// ------ COMMON RE-EXPORTS

pub use crate::attributes::{AttributeBind, AttributeUpdate};
pub use crate::cmap::{
    BuilderError, CMap2, CMapBuilder, DartIdentifier, EdgeIdentifier, FaceIdentifier, Orbit2,
    OrbitPolicy, VertexIdentifier, VolumeIdentifier, NULL_DART_ID, NULL_EDGE_ID, NULL_FACE_ID,
    NULL_VERTEX_ID, NULL_VOLUME_ID,
};
pub use crate::geometry::{CoordsError, CoordsFloat, Vector2, Vertex2};

// ------ FEATURE-GATED RE-EXPORTS

#[cfg(feature = "utils")]
pub use crate::cmap::GridDescriptor;
