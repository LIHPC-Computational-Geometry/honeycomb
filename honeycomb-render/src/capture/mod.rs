pub mod ecs_data;
mod system;

use crate::capture::ecs_data::CaptureId;
use crate::{DartBodyBundle, DartHeadBundle, EdgeBundle, FaceBundle, VertexBundle};
use bevy::prelude::*;
use bevy::utils::HashMap;
use honeycomb_core::{CMap2, CoordsFloat, FaceIdentifier};

#[derive(Resource)]
pub struct FocusedCapture(pub CaptureId);

#[derive(Resource)]
pub struct CaptureList(pub Vec<Capture>);

pub struct Capture {
    pub metadata: CaptureMD,
    pub vertex_vals: Vec<Vec3>,
    pub normals: HashMap<FaceIdentifier, Vec<Vec3>>,
    pub darts: Vec<(DartHeadBundle, DartBodyBundle)>,
    pub vertices: Vec<VertexBundle>,
    pub edges: Vec<EdgeBundle>,
    pub faces: Vec<FaceBundle>,
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
