//! combinatorial maps implementations

mod collections;
mod dim2;

pub use collections::{
    EdgeCollection, EdgeIdentifier, FaceCollection, FaceIdentifier, VertexCollection,
    VertexIdentifier, VolumeIdentifier, NULL_EDGE_ID, NULL_FACE_ID, NULL_VERTEX_ID, NULL_VOLUME_ID,
};
pub use dim2::{
    orbits::{Orbit2, OrbitPolicy},
    structure::CMap2,
};
