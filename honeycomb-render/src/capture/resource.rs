use bevy::prelude::*;
use honeycomb_core::{DartIdentifier, EdgeIdentifier, FaceIdentifier, VertexIdentifier};

#[derive(Component)]
pub struct DartHead {
    pub map_id: DartIdentifier,
    pub vertex: usize,                   // (v0_id, v1_id)
    pub normal: (FaceIdentifier, usize), // vertex normals (for shrink ops)
}

#[derive(Component)]
pub struct DartBody {
    pub map_id: DartIdentifier,
    pub vertices: (usize, usize), // (v0_id, v1_id)
    pub normals: ((FaceIdentifier, usize), (FaceIdentifier, usize)), // vertex normals (for shrink ops)
}

#[derive(Component)]
pub struct Beta(pub u8, pub usize, pub usize); // beta id, v0_id, v1_id ?

#[derive(Component)]
pub struct Vertex(pub VertexIdentifier, pub usize); // map id, vid

#[derive(Component)]
pub struct Edge(pub EdgeIdentifier, pub usize, pub usize); // map id, v0_id, v1_id

#[derive(Component)]
pub struct Face {
    pub map_id: FaceIdentifier,
    pub vertices: Vec<usize>,                  // vertex list
    pub normals: Vec<(FaceIdentifier, usize)>, // vertex normal list (for shrink ops)
}

#[derive(Component)]
pub struct Volume;
