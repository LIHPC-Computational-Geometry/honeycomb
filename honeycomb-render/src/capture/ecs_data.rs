use bevy::prelude::*;
use bevy::utils::HashMap;
use honeycomb_core::{DartIdentifier, EdgeIdentifier, FaceIdentifier, VertexIdentifier};

// --- shared data

#[derive(Resource)]
pub struct MapVertices(pub Vec<Vec3>);

#[derive(Resource)]
pub struct FaceNormals(pub HashMap<FaceIdentifier, Vec<Vec3>>);

// --- bundles

#[derive(Bundle, Clone)]
pub struct DartHeadBundle {
    pub capture_id: CaptureId,
    id: MapId<DartIdentifier>,
    vertex_id: MapId<VertexIdentifier>,
    edge_id: MapId<EdgeIdentifier>,
    pub face_id: MapId<FaceIdentifier>,
    pub dart_head: DartHead,
}

impl DartHeadBundle {
    pub fn new(
        capture_id: usize,
        id: DartIdentifier,
        vertex_id: VertexIdentifier,
        edge_id: EdgeIdentifier,
        face_id: FaceIdentifier,
        vertex: usize,
        normal: usize,
    ) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: MapId(id),
            vertex_id: MapId(vertex_id),
            edge_id: MapId(edge_id),
            face_id: MapId(face_id),
            dart_head: DartHead { vertex, normal },
        }
    }
}

#[derive(Bundle, Clone)]
pub struct DartBodyBundle {
    pub capture_id: CaptureId,
    id: MapId<DartIdentifier>,
    vertex_id: MapId<VertexIdentifier>,
    edge_id: MapId<EdgeIdentifier>,
    pub face_id: MapId<FaceIdentifier>,
    pub dart_body: DartBody,
}

impl DartBodyBundle {
    pub fn new(
        capture_id: usize,
        id: DartIdentifier,
        vertex_id: VertexIdentifier,
        edge_id: EdgeIdentifier,
        face_id: FaceIdentifier,
        vertices: (usize, usize),
        normals: (usize, usize),
    ) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: MapId(id),
            vertex_id: MapId(vertex_id),
            edge_id: MapId(edge_id),
            face_id: MapId(face_id),
            dart_body: DartBody { vertices, normals },
        }
    }
}

#[derive(Bundle, Clone)]
pub struct VertexBundle {
    pub capture_id: CaptureId,
    id: MapId<VertexIdentifier>,
    pub vertex: Vertex,
}

impl VertexBundle {
    pub fn new(capture_id: usize, id: VertexIdentifier, vertex: usize) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: MapId(id),
            vertex: Vertex(vertex),
        }
    }
}

#[derive(Bundle, Clone)]
pub struct EdgeBundle {
    pub capture_id: CaptureId,
    id: MapId<EdgeIdentifier>,
    pub edge: Edge,
}

impl EdgeBundle {
    pub fn new(capture_id: usize, id: EdgeIdentifier, vertices: (usize, usize)) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: MapId(id),
            edge: Edge(vertices.0, vertices.1),
        }
    }
}

#[derive(Bundle, Clone)]
pub struct FaceBundle {
    pub capture_id: CaptureId,
    pub id: MapId<FaceIdentifier>,
    pub face: Face,
}

impl FaceBundle {
    pub fn new(capture_id: usize, id: FaceIdentifier, vertices: Vec<usize>) -> Self {
        Self {
            capture_id: CaptureId(capture_id),
            id: MapId(id),
            face: Face(vertices),
        }
    }
}

// --- individual components

#[derive(Component, Clone, PartialEq, Eq)]
pub struct CaptureId(pub usize);

#[derive(Component, Clone)]
pub struct MapId<I>(pub I);

#[derive(Component, Clone)]
pub struct DartHead {
    pub vertex: usize, // (v0_id, v1_id)
    pub normal: usize, // vertex normals (for shrink ops)
}

#[derive(Component, Clone)]
pub struct DartBody {
    pub vertices: (usize, usize), // (v0_id, v1_id)
    pub normals: (usize, usize),  // vertex normals (for shrink ops)
}

#[derive(Component, Clone)]
pub struct Beta(pub u8, pub usize, pub usize); // beta id, v0_id, v1_id ?

#[derive(Component, Clone)]
pub struct Vertex(pub usize); // map id, vid

#[derive(Component, Clone)]
pub struct Edge(pub usize, pub usize); // v0_id, v1_id

#[derive(Component, Clone)]
pub struct Face(pub Vec<usize>); // vertex list

#[derive(Component, Clone)]
pub struct Volume;
