mod resource;

use bevy::prelude::*;
use bevy::utils::HashMap;
use honeycomb_core::{CMap2, CoordsFloat, FaceIdentifier};

pub struct Capture {
    pub metadata: CaptureMD,
    pub vertices: Vec<Vec2>,
    pub normals: HashMap<FaceIdentifier, Vec<Vec2>>,
}

impl<T: CoordsFloat> From<&CMap2<T>> for Capture {
    fn from(_: &CMap2<T>) -> Self {
        todo!()
    }
}

pub struct CaptureMD {
    pub capture_id: usize,
    pub n_darts: usize,
    pub n_vertices: usize,
    pub n_edges: usize,
    pub n_faces: usize,
    pub n_volumes: usize,
}
