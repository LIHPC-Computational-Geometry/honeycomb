use honeycomb_core::{CMap2, CoordsFloat};

pub struct Capture {
    pub metadata: CaptureMD,
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
