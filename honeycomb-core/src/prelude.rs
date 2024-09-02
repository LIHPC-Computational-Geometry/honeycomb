// ------ COMMON RE-EXPORTS

pub use crate::attributes::traits::{AttributeBind, AttributeUpdate};
pub use crate::cmap::{
    BuilderError, CMap2, CMapBuilder, CMapError, DartIdentifier, EdgeIdentifier, FaceIdentifier,
    Orbit2, OrbitPolicy, VertexIdentifier, VolumeIdentifier, NULL_DART_ID, NULL_EDGE_ID,
    NULL_FACE_ID, NULL_VERTEX_ID, NULL_VOLUME_ID,
};
pub use crate::spatial_repr::{Vector2, Vertex2};

// ------ FEATURE-GATED RE-EXPORTS

#[cfg(feature = "utils")]
pub use crate::cmap::GridDescriptor;
