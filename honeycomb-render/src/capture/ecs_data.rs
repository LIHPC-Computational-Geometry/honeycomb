use bevy::prelude::*;
use bevy::utils::HashMap;
use honeycomb_core::prelude::{DartIdType, EdgeIdType, FaceIdType, VertexIdType, VolumeIdType};

// --- shared data

/// Collection of a map's vertices.
#[derive(Resource)]
pub struct MapVertices(pub Vec<Vec3>);

/// Collection of normals, organized per faces of a map.
#[derive(Resource)]
pub struct FaceNormals(pub HashMap<(FaceIdType, usize), Vec3>);

/// Collection of normals, organized per volumes of a map.
#[derive(Resource)]
pub struct VolumeNormals(pub HashMap<(VolumeIdType, usize), Vec3>);

// --- bundles

/// Bundle used to create entities modeling dart bodies.
#[derive(Bundle, Clone)]
pub struct DartBundle {
    pub(crate) id: DartId,
    pub(crate) vertex_id: VertexId,
    pub(crate) edge_id: EdgeId,
    pub(crate) face_id: FaceId,
    pub(crate) dart: Dart,
}

/// Bundle used to create entities modeling vertices.
#[derive(Bundle, Clone)]
pub struct VertexBundle {
    pub(crate) id: VertexId,
    pub(crate) vertex: Vertex,
}

/// Bundle used to create entities modeling edges.
#[derive(Bundle, Clone)]
pub struct EdgeBundle {
    pub(crate) id: EdgeId,
    pub(crate) edge: Edge,
}

/// Bundle used to create entities modeling faces.
#[derive(Bundle, Clone)]
pub struct FaceBundle {
    pub(crate) id: FaceId,
    pub(crate) face: Face,
}

// --- individual components

/// Dart ID component.
#[derive(Component, Clone)]
pub struct DartId(pub DartIdType);

/// Vertex ID component.
#[derive(Component, Clone)]
pub struct VertexId(pub VertexIdType);

/// Edge ID component.
#[derive(Component, Clone)]
pub struct EdgeId(pub EdgeIdType);

/// Face ID component.
#[derive(Component, Clone)]
pub struct FaceId(pub FaceIdType);

/// Volume ID component.
#[derive(Component, Clone)]
pub struct VolumeId(pub VolumeIdType);

/// Dart head component.
#[derive(Component, Clone)]
pub struct Dart {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// Beta component.
#[derive(Component, Clone)]
pub struct Beta(pub u8, pub usize, pub usize); // beta id, v0_id, v1_id ?

/// Vertex component.
#[derive(Component, Clone)]
pub struct Vertex(pub usize); // map id, vid

/// Edge component.
#[derive(Component, Clone)]
pub struct Edge(pub usize, pub usize); // v0_id, v1_id

/// Face component.
#[derive(Component, Clone)]
pub struct Face(pub Vec<usize>); // vertex list

/// Volume component.
#[derive(Component, Clone)]
pub struct Volume;
