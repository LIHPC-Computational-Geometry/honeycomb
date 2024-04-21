//! map - rendering data interface code
//!
//! This module contains all the code used to convert data read from the map reference to
//! data that can be interpreted and rendered by the shader system.

// ------ IMPORTS

use crate::shader_data::Coords2Shader;
use crate::SmaaMode;
use honeycomb_core::{CMap2, CoordsFloat, DartIdentifier, Vertex2};
use std::iter::zip;

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

pub struct CMap2RenderHandle<'a, T: CoordsFloat> {
    handle: &'a CMap2<T>,
    params: RenderParameters,
    intermediate_buffer: Vec<Vertex2<T>>,
    dart_construction_buffer: Vec<Coords2Shader>,
    _beta_construction_buffer: Vec<Coords2Shader>,
    face_construction_buffer: Vec<Coords2Shader>,
    vertices: Vec<Coords2Shader>,
}

impl<'a, T: CoordsFloat> CMap2RenderHandle<'a, T> {
    pub fn new(map: &'a CMap2<T>, params: Option<RenderParameters>) -> Self {
        Self {
            handle: map,
            params: params.unwrap_or_default(),
            intermediate_buffer: Vec::new(),
            dart_construction_buffer: Vec::new(),
            _beta_construction_buffer: Vec::new(),
            face_construction_buffer: Vec::new(),
            vertices: Vec::new(),
        }
    }

    fn build_intermediate(&mut self) {
        let faces = self.handle.fetch_faces();
        let face_indices = faces.identifiers.iter();

        todo!()
    }

    pub fn build_darts(&mut self) {
        // get all faces
        let faces = self.handle.fetch_faces();
        let face_indices = faces.identifiers.iter();
        let face_vertices = face_indices.clone().map(|face_id| {
            (
                self.handle.i_cell::<2>(*face_id as DartIdentifier).count(),
                self.handle
                    .i_cell::<2>(*face_id as DartIdentifier)
                    .map(|dart_id| {
                        let vertex_id = self.handle.vertex_id(dart_id);
                        self.handle.vertex(vertex_id)
                    }),
            )
        });
        let centers = face_vertices.clone().map(|(n_vertices, vertices)| {
            vertices
                .map(|vertex2: Vertex2<T>| vertex2.into_inner())
                .reduce(|coords1, coords2| coords1 + coords2)
                .unwrap()
                / T::from(n_vertices).unwrap()
        });
        self.dart_construction_buffer.extend(
            face_indices
                .copied()
                .zip(face_vertices)
                .zip(centers)
                .flat_map(|((face_id, (n_vertices, vertices)), center)| {
                    let vertices: Vec<Vertex2<T>> = vertices.collect();
                    let vertices_pair = (0..n_vertices).map(move |vid| {
                        let v1id = vid;
                        let v2id = if vid == n_vertices - 1 { 0 } else { vid + 1 };
                        (vertices[v1id], vertices[v2id])
                    });
                    let metadata = (0..n_vertices).map(move |_| (face_id, Vertex2::from(center)));
                    vertices_pair
                        .zip(metadata)
                        .map(|((v1, v2), (fid, centerv))| {
                            // shrink towards center
                            let v1_shrink_dir = (centerv - v1).unit_dir().unwrap();
                            let v2_shrink_dir = (centerv - v2).unit_dir().unwrap();

                            let mut v1_intermediate =
                                v1 + v1_shrink_dir * T::from(self.params.shrink_factor).unwrap();
                            let mut v2_intermediate =
                                v2 + v2_shrink_dir * T::from(self.params.shrink_factor).unwrap();

                            // truncate length
                            let seg_dir = (v2_intermediate - v1_intermediate).unit_dir().unwrap();
                            v1_intermediate +=
                                seg_dir * T::from(self.params.shrink_factor).unwrap();
                            v2_intermediate -=
                                seg_dir * T::from(self.params.shrink_factor).unwrap();

                            // return a coordinate pair
                            (v1_intermediate, v2_intermediate, fid)
                        })
                })
                .flat_map(|(v1, v6, face_id)| {
                    // transform the dart coordinates into triangles for the shader to render
                    let seg = v6 - v1;
                    let seg_length = seg.norm();
                    let seg_dir = seg.unit_dir().unwrap();
                    let seg_normal = seg.normal_dir();
                    let ahs = T::from(self.params.arrow_headsize).unwrap();
                    let at = T::from(self.params.arrow_thickness).unwrap();

                    let vcenter = v6 - seg_dir * ahs;
                    let v1 = v1.into_inner();
                    let v2 = (vcenter - seg_normal * at).into_inner();
                    let v3 = (vcenter + seg_normal * at).into_inner();
                    let v4 = (vcenter + seg_normal * (ahs * seg_length)).into_inner();
                    let v5 = (vcenter - seg_normal * (ahs * seg_length)).into_inner();
                    let v6 = v6.into_inner();

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

    pub fn build_faces(&mut self) {
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
