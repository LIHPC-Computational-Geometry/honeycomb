//! map - rendering data interface code
//!
//! This module contains all the code used to convert data read from the map reference to
//! data that can be interpreted and rendered by the shader system.

// ------ IMPORTS

use crate::representations::intermediates::{Entity, IntermediateFace};
use crate::shader_data::Coords2Shader;
use crate::SmaaMode;
use honeycomb_core::{CMap2, CoordsFloat, DartIdentifier, Orbit2, OrbitPolicy};

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

pub struct CMap2RenderHandle<'a, T: CoordsFloat> {
    handle: &'a CMap2<T>,
    params: RenderParameters,
    intermediate_buffer: Vec<IntermediateFace<T>>,
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

    pub(crate) fn build_intermediate(&mut self) {
        let faces = self.handle.fetch_faces();
        let faces_ir = faces.identifiers.iter().map(|face_id| {
            // build face data
            let orbit = Orbit2::new(self.handle, OrbitPolicy::Face, *face_id as DartIdentifier)
                .map(|id| self.handle.vertex(self.handle.vertex_id(id)));
            let mut tmp = IntermediateFace::new(orbit);
            // apply a first shrink
            tmp.vertices.iter_mut().for_each(|v| {
                let v_shrink_dir = (tmp.center - *v).unit_dir().unwrap();
                *v += v_shrink_dir * T::from(self.params.shrink_factor).unwrap();
            });
            tmp
        });
        // save results
        self.intermediate_buffer.extend(faces_ir);
    }

    pub fn build_darts(&mut self) {
        // get all faces
        let tmp = self.intermediate_buffer.iter().flat_map(|face| {
            (0..face.n_vertices).flat_map(|id| {
                let mut v1 = face.vertices[id];
                let mut v6 = face.vertices[(id + 1) % face.n_vertices];

                let seg_dir = (v6 - v1).unit_dir().unwrap();
                v1 += seg_dir * T::from(self.params.shrink_factor).unwrap();
                v6 -= seg_dir * T::from(self.params.shrink_factor).unwrap();

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
                    Coords2Shader::from((v1, Entity::Dart)),
                    Coords2Shader::from((v2, Entity::Dart)),
                    Coords2Shader::from((v3, Entity::Dart)),
                    Coords2Shader::from((v4, Entity::Dart)),
                    Coords2Shader::from((v5, Entity::Dart)),
                    Coords2Shader::from((v6, Entity::Dart)),
                ]
                .into_iter()
            })
        });
        self.dart_construction_buffer.extend(tmp);
    }

    #[allow(dead_code)]
    pub fn build_betas(&mut self) {
        todo!()
    }

    pub fn build_faces(&mut self) {
        // because there's no trianglefan priitive in the webgpu standard,
        // we have to duplicate vertices
        let tmp = self.intermediate_buffer.iter().flat_map(|face| {
            (1..face.n_vertices - 1).flat_map(|id| {
                let mut tmp1 = face.vertices[0];
                let mut tmp2 = face.vertices[id];
                let mut tmp3 = face.vertices[id + 1];
                let shrink_dir1 = (face.center - tmp1).unit_dir().unwrap();
                let shrink_dir2 = (face.center - tmp2).unit_dir().unwrap();
                let shrink_dir3 = (face.center - tmp3).unit_dir().unwrap();
                tmp1 += shrink_dir1 * T::from(self.params.shrink_factor * 2.0).unwrap();
                tmp2 += shrink_dir2 * T::from(self.params.shrink_factor * 2.0).unwrap();
                tmp3 += shrink_dir3 * T::from(self.params.shrink_factor * 2.0).unwrap();
                [
                    Coords2Shader::from((tmp1, Entity::Face)),
                    Coords2Shader::from((tmp2, Entity::Face)),
                    Coords2Shader::from((tmp3, Entity::Face)),
                ]
                .into_iter()
            })
        });
        self.face_construction_buffer.extend(tmp);
    }

    pub fn save_buffered(&mut self) {
        self.vertices.clear();
        self.vertices.append(&mut self.dart_construction_buffer);
        self.vertices.append(&mut self.face_construction_buffer);
    }

    pub fn vertices(&self) -> &[Coords2Shader] {
        &self.vertices
    }
}
