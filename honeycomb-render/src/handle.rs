//! map - rendering data interface code
//!
//! This module contains all the code used to convert data read from the map reference to
//! data that can be interpreted and rendered by the shader system.

// ------ IMPORTS

use crate::shader_data::Coords2Shader;
use crate::SmaaMode;
use honeycomb_core::{Coords2, CoordsFloat, FaceIdentifier, TwoMap};

// ------ CONTENT

/// Rendering parameters encapsulation
///
/// The user can easily adjust parameters in this structure to obtain
/// the preferred output style.
pub struct RenderParameters {
    /// Anti-aliasing configuration.
    pub smaa_mode: SmaaMode,
    /// Shrink factor used to compute dart position from edge position.
    pub shrink_factor: f32,
    /// Size of the dart head (related to its length).
    pub arrow_headsize: f32,
    /// Thickness of the darts.
    pub arrow_thickness: f32,
}

impl Default for RenderParameters {
    fn default() -> Self {
        Self {
            smaa_mode: SmaaMode::Disabled,
            shrink_factor: 0.1,    // need to adjust
            arrow_headsize: 0.1,   // need to adjust
            arrow_thickness: 0.01, // need to adjust
        }
    }
}

macro_rules! as_f32_tuple {
    ($coords: ident) => {
        ($coords.x.to_f32().unwrap(), $coords.y.to_f32().unwrap())
    };
}

pub struct TwoMapRenderHandle<'a, const N_MARKS: usize, T: CoordsFloat> {
    handle: &'a TwoMap<N_MARKS, T>,
    params: RenderParameters,
    dart_construction_buffer: Vec<Coords2Shader>,
    _beta_construction_buffer: Vec<Coords2Shader>,
    vertices: Vec<Coords2Shader>,
}

impl<'a, const N_MARKS: usize, T: CoordsFloat> TwoMapRenderHandle<'a, N_MARKS, T> {
    pub fn new(map: &'a TwoMap<N_MARKS, T>, params: Option<RenderParameters>) -> Self {
        Self {
            handle: map,
            params: params.unwrap_or_default(),
            dart_construction_buffer: Vec::new(),
            _beta_construction_buffer: Vec::new(),
            vertices: Vec::new(),
        }
    }

    pub fn build_darts(&mut self) {
        let n_face = self.handle.n_faces() as FaceIdentifier;
        let face_iter = (0..n_face).map(|face_id| {
            let cell = self.handle.face(face_id);
            // compute face center for shrink operation
            let center: Coords2<T> = cell
                .corners
                .iter()
                .map(|vid| self.handle.vertex(*vid))
                .sum::<Coords2<T>>()
                / T::from(cell.corners.len()).unwrap();
            (cell, center, face_id)
        });
        self.dart_construction_buffer.extend(
            face_iter
                .flat_map(|(cell, center, face_id)| {
                    let n_vertices = cell.corners.len();
                    let fids = (0..n_vertices).map(move |_| face_id);
                    let centers = (0..n_vertices).map(move |_| center);
                    (0..n_vertices)
                        .zip(centers)
                        .map(|(vertex_id, center)| {
                            let v1id = vertex_id;
                            let v2id = if vertex_id == cell.corners.len() - 1 {
                                0
                            } else {
                                vertex_id + 1
                            };
                            // fetch dart vetices
                            let (v1, v2) = (
                                self.handle.vertex(cell.corners[v1id]),
                                self.handle.vertex(cell.corners[v2id]),
                            );

                            // shrink towards center
                            let v1_shrink_dir = (center - *v1).unit_dir().unwrap();
                            let v2_shrink_dir = (center - *v2).unit_dir().unwrap();

                            let mut v1_intermediate =
                                *v1 + v1_shrink_dir * T::from(self.params.shrink_factor).unwrap();
                            let mut v2_intermediate =
                                *v2 + v2_shrink_dir * T::from(self.params.shrink_factor).unwrap();

                            // truncate length
                            let seg_dir = (v2_intermediate - v1_intermediate).unit_dir().unwrap();
                            v1_intermediate +=
                                seg_dir * T::from(self.params.shrink_factor).unwrap();
                            v2_intermediate -=
                                seg_dir * T::from(self.params.shrink_factor).unwrap();

                            // return a coordinate pair
                            (v1_intermediate, v2_intermediate)
                        })
                        .zip(fids)
                })
                .flat_map(|((v1, v6), face_id)| {
                    // transform the dart coordinates into triangles for the shader to render
                    let seg = v6 - v1;
                    let seg_length = seg.norm();
                    let seg_dir = seg.unit_dir().unwrap();
                    let seg_normal = seg.normal_dir();
                    let ahs = T::from(self.params.arrow_headsize).unwrap();
                    let at = T::from(self.params.arrow_thickness).unwrap();

                    let vcenter = v6 - seg_dir * ahs;
                    let v2 = vcenter - seg_normal * at;
                    let v3 = vcenter + seg_normal * at;
                    let v4 = vcenter + seg_normal * (ahs * seg_length);
                    let v5 = vcenter - seg_normal * (ahs * seg_length);

                    [
                        Coords2Shader::new(as_f32_tuple!(v1), face_id),
                        Coords2Shader::new(as_f32_tuple!(v2), face_id),
                        Coords2Shader::new(as_f32_tuple!(v3), face_id),
                        Coords2Shader::new(as_f32_tuple!(v4), face_id),
                        Coords2Shader::new(as_f32_tuple!(v5), face_id),
                        Coords2Shader::new(as_f32_tuple!(v6), face_id),
                    ]
                    .into_iter()
                }),
        );
    }

    #[allow(dead_code)]
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
