//!

pub use honeycomb_core as core;

#[cfg(feature = "kernels")]
pub use honeycomb_kernels as kernels;

#[cfg(feature = "render")]
pub use honeycomb_render as render;

pub mod prelude {
    // ------ CORE RE-EXPORTS

    pub use honeycomb_core::attributes::{AttributeBind, AttributeUpdate};
    pub use honeycomb_core::cmap::{
        BuilderError, CMap2, CMapBuilder, CMapError, DartIdentifier, EdgeIdentifier,
        FaceIdentifier, GridDescriptor, Orbit2, OrbitPolicy, VertexIdentifier, VolumeIdentifier,
        NULL_DART_ID, NULL_EDGE_ID, NULL_FACE_ID, NULL_VERTEX_ID, NULL_VOLUME_ID,
    };
    pub use honeycomb_core::geometry::{CoordsError, CoordsFloat, Vector2, Vertex2};

    // ------ KERNELS RE-EXPORTS

    #[cfg(feature = "kernels")]
    pub use honeycomb_kernels::grisubal;

    // ------ RENDER RE-EXPORTS

    #[cfg(feature = "render")]
    pub use honeycomb_render::App;
}
