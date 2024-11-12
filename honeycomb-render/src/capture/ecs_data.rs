use bevy::prelude::*;
use bevy::utils::HashMap;
use honeycomb_core::prelude::{DartIdType, EdgeIdType, FaceIdType, VertexIdType, VolumeIdType};

// --- shared data

/// Collection of a map's vertices.
#[derive(Resource)]
pub struct MapVertices(pub Vec<Vec3>);

/// Collection of normals, organized per faces of a map.
#[derive(Resource)]
pub struct FaceNormals(pub HashMap<FaceIdType, Vec<Vec3>>);

// --- bundles

/// Bundle used to create entities modeling dart heads.
#[derive(Bundle, Clone)]
pub struct DartHeadBundle {
    pub(crate) capture_id: CaptureId,
    id: DartId,
    vertex_id: VertexId,
    edge_id: EdgeId,
    pub(crate) face_id: FaceId,
    pub(crate) dart_head: DartHead,
}

impl DartHeadBundle {
    /// Constructor.
    #[must_use = "Object unused after construction"]
    pub fn new(
        capture_id: usize,
        id: DartIdType,
        vertex_id: VertexIdType,
        edge_id: EdgeIdType,
        face_id: FaceIdType,
        vertices: (usize, usize),
        normals: (usize, usize),
    ) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: DartId(id),
            vertex_id: VertexId(vertex_id),
            edge_id: EdgeId(edge_id),
            face_id: FaceId(face_id),
            dart_head: DartHead { vertices, normals },
        }
    }
}

/// Bundle used to create entities modeling dart bodies.
#[derive(Bundle, Clone)]
pub struct DartBodyBundle {
    pub(crate) capture_id: CaptureId,
    id: DartId,
    vertex_id: VertexId,
    edge_id: EdgeId,
    pub(crate) face_id: FaceId,
    pub(crate) dart_body: DartBody,
}

impl DartBodyBundle {
    /// Constructor.
    #[must_use = "Object unused after construction"]
    pub fn new(
        capture_id: usize,
        id: DartIdType,
        vertex_id: VertexIdType,
        edge_id: EdgeIdType,
        face_id: FaceIdType,
        vertices: (usize, usize),
        normals: (usize, usize),
    ) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: DartId(id),
            vertex_id: VertexId(vertex_id),
            edge_id: EdgeId(edge_id),
            face_id: FaceId(face_id),
            dart_body: DartBody { vertices, normals },
        }
    }
}

/// Bundle used to create entities modeling vertices.
#[derive(Bundle, Clone)]
pub struct VertexBundle {
    pub(crate) capture_id: CaptureId,
    id: VertexId,
    pub(crate) vertex: Vertex,
}

impl VertexBundle {
    /// Constructor.
    #[must_use = "Object unused after construction"]
    pub fn new(capture_id: usize, id: VertexIdType, vertex: usize) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: VertexId(id),
            vertex: Vertex(vertex),
        }
    }
}

/// Bundle used to create entities modeling edges.
#[derive(Bundle, Clone)]
pub struct EdgeBundle {
    pub(crate) capture_id: CaptureId,
    id: EdgeId,
    pub(crate) edge: Edge,
}

impl EdgeBundle {
    /// Constructor.
    #[must_use = "Object unused after construction"]
    pub fn new(capture_id: usize, id: EdgeIdType, vertices: (usize, usize)) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: EdgeId(id),
            edge: Edge(vertices.0, vertices.1),
        }
    }
}

/// Bundle used to create entities modeling faces.
#[derive(Bundle, Clone)]
pub struct FaceBundle {
    pub(crate) capture_id: CaptureId,
    pub(crate) id: FaceId,
    pub(crate) face: Face,
}

impl FaceBundle {
    /// Constructor.
    #[must_use = "Object unused after construction"]
    pub fn new(capture_id: usize, id: FaceIdType, vertices: Vec<usize>) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: FaceId(id),
            face: Face(vertices),
        }
    }
}

// --- individual components

/// Capture ID component.
#[derive(Component, Clone, PartialEq, Eq)]
pub struct CaptureId(pub usize);

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
pub struct DartHead {
    pub(crate) vertices: (usize, usize), // (v0_id, v1_id); we need both for rotation computations
    pub(crate) normals: (usize, usize),  // vertex normals (for shrink ops)
}

/// Dart body component.
#[derive(Component, Clone)]
pub struct DartBody {
    pub(crate) vertices: (usize, usize), // (v0_id, v1_id)
    pub(crate) normals: (usize, usize),  // vertex normals (for shrink ops)
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
