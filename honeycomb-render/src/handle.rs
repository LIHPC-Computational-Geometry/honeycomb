//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::shader_data::Coords2Shader;
use crate::SmaaMode;
use honeycomb_core::{Coords2, CoordsFloat, FaceIdentifier, TwoMap};

// ------ CONTENT

pub struct RenderParameters {
    pub smaa_mode: SmaaMode,
    pub shrink_factor: f32,
    pub arrow_headsize: f32,
    pub arrow_thickness: f32,
}

impl Default for RenderParameters {
    fn default() -> Self {
        Self {
            smaa_mode: SmaaMode::Disabled,
            shrink_factor: 0.1,   // need to adjust
            arrow_headsize: 0.1,  // need to adjust
            arrow_thickness: 0.1, // need to adjust
        }
    }
}

pub struct TwoMapRenderHandle<'a, const N_MARKS: usize, T: CoordsFloat> {
    handle: &'a TwoMap<N_MARKS, T>,
    params: RenderParameters,
    dart_construction_buffer: Vec<Coords2Shader>,
    beta_construction_buffer: Vec<Coords2Shader>,
    vertices: Vec<Coords2Shader>,
}

impl<'a, const N_MARKS: usize, T: CoordsFloat> TwoMapRenderHandle<'a, N_MARKS, T> {
    pub fn new(map: &'a TwoMap<N_MARKS, T>, params: Option<RenderParameters>) -> Self {
        Self {
            handle: map,
            params: params.unwrap_or_default(),
            dart_construction_buffer: Vec::new(),
            beta_construction_buffer: Vec::new(),
            vertices: Vec::new(),
        }
    }

    pub fn build_darts(&mut self) {
        let n_face = self.handle.n_faces() as FaceIdentifier;
        self.dart_construction_buffer.extend(
            (0..n_face)
                .flat_map(|face_id| {
                    let cell = self.handle.face(face_id);
                    // compute face center for shrink operation
                    let center: Coords2<T> = cell
                        .corners
                        .iter()
                        .map(|vid| self.handle.vertex(*vid))
                        .sum::<Coords2<T>>()
                        / T::from(cell.corners.len()).unwrap();
                    let n_vertices = cell.corners.len();
                    let fids = (0..n_vertices - 1).map(move |_| face_id);
                    (0..n_vertices - 1)
                        .map(|vertex_id| {
                            // fetch dart vetices
                            let (mut v1, mut v2) = (
                                self.handle.vertex(cell.corners[vertex_id]),
                                self.handle.vertex(cell.corners[vertex_id + 1]),
                            );
                            // shrink

                            // return a coordinate pair
                            (v1, v2)
                        })
                        .zip(fids)
                })
                .flat_map(|((v1, v2), face_id)| {
                    // transform the coordinates into triangles for the shader to render

                    [
                        Coords2Shader::new((0.0, 0.0), 0),
                        Coords2Shader::new((0.0, 0.0), 0),
                    ]
                    .into_iter()
                }),
        );
    }

    pub fn build_betas(&mut self) {
        todo!()
    }

    pub fn save_buffered(&mut self) {
        self.vertices.clear();
        self.vertices.append(&mut self.dart_construction_buffer);
    }

    pub fn vertices(&self) -> &[Coords2Shader] {
        &self.vertices
    }
}
