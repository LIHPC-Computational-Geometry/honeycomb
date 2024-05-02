//! map - rendering data interface code
//!
//! This module contains all the code used to convert data read from the map reference to
//! data that can be interpreted and rendered by the shader system.

// ------ IMPORTS

use crate::representations::intermediates::{Entity, IntermediateFace};
use crate::shader_data::Coords2Shader;
use crate::SmaaMode;
use honeycomb_core::{
    CMap2, CoordsFloat, DartIdentifier, EdgeIdentifier, Orbit2, OrbitPolicy, Vertex2, NULL_DART_ID,
};

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
    /// Size arrow thcikness and head relatively to their length.
    pub relative_resize: bool,
}

impl Default for RenderParameters {
    fn default() -> Self {
        Self {
            smaa_mode: SmaaMode::Disabled,
            shrink_factor: 0.1,    // need to adjust
            arrow_headsize: 0.05,  // need to adjust
            arrow_thickness: 0.01, // need to adjust
            relative_resize: true,
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
                .map(|id| {
                    // in order to render the map, all vertices of a given face should be defined
                    // unwraping here will crash the program, telling the user that a vertex could
                    // not be found
                    self.handle.vertex(self.handle.vertex_id(id)).unwrap()
                });
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
        let tmp = self
            .intermediate_buffer
            .iter()
            .filter(|face| face.n_vertices > 1)
            .flat_map(|face| {
                (0..face.n_vertices).flat_map(|id| {
                    let mut va = face.vertices[id];
                    let mut vb = face.vertices[(id + 1) % face.n_vertices];

                    // if vb == va, this unwrap will panick
                    // this is ok ATM since there should not be two perfectly overlapping
                    // darts/vertices; This would indicate that the face is incorrectly built
                    let seg_dir = (vb - va).unit_dir().unwrap();
                    va += seg_dir * T::from(self.params.shrink_factor).unwrap();
                    vb -= seg_dir * T::from(self.params.shrink_factor).unwrap();

                    let seg = vb - va;
                    // same as above;
                    // but the superposition could also be induced by an invalid shrink value here
                    let seg_normal = seg.normal_dir().unwrap();
                    let ahs = T::from(self.params.arrow_headsize).unwrap();
                    let at = T::from(self.params.arrow_thickness).unwrap();
                    let mut body_offset = seg_normal * at;
                    let mut head_offset = seg_normal * ahs;
                    let mut vcenter = vb - seg * ahs;
                    if self.params.relative_resize {
                        let seg_length = seg.norm();
                        body_offset *= seg_length;
                        head_offset *= seg_length;
                        vcenter = vb - seg * seg_length * ahs;
                    }

                    let v1 = va + body_offset;
                    let v2 = va - body_offset;
                    let v3 = vcenter - body_offset;
                    let v4 = v3;
                    let v5 = v1;
                    let v6 = vcenter + body_offset;
                    let v7 = vcenter + head_offset;
                    let v8 = vcenter - head_offset;
                    let v9 = vb;
                    [
                        Coords2Shader::from((v1, Entity::Dart)),
                        Coords2Shader::from((v2, Entity::Dart)),
                        Coords2Shader::from((v3, Entity::Dart)),
                        Coords2Shader::from((v4, Entity::Dart)),
                        Coords2Shader::from((v5, Entity::Dart)),
                        Coords2Shader::from((v6, Entity::Dart)),
                        Coords2Shader::from((v7, Entity::Dart)),
                        Coords2Shader::from((v8, Entity::Dart)),
                        Coords2Shader::from((v9, Entity::Dart)),
                    ]
                    .into_iter()
                })
            });
        self.dart_construction_buffer.extend(tmp);
    }

    #[allow(dead_code)]
    pub fn build_betas(&mut self) {
        let tmp: Vec<EdgeIdentifier> = self.handle.fetch_edges().identifiers.clone();
        let tmp = tmp
            .iter()
            .map(|edge_id| {
                (
                    *edge_id as DartIdentifier,
                    self.handle.beta::<2>(*edge_id as DartIdentifier),
                )
            })
            .filter(|(_, b2vid)| *b2vid != NULL_DART_ID)
            .flat_map(|(dart_id, b2dart_id)| {
                // in order to render the beta functions,
                // the two vertices of each given edge in required
                // dart that are free by beta 2 were already filtered above, so the
                // only ones left should have a valid definition for the render to proceed
                let va = self.handle.vertex(self.handle.vertex_id(dart_id)).unwrap();
                let vb = self
                    .handle
                    .vertex(self.handle.vertex_id(b2dart_id))
                    .unwrap();
                let seg_dir = vb - va;
                let center = Vertex2::average(&va, &vb);
                // if vb == va, this unwrap will panick
                // this is ok ATM since there should not be two perfectly overlapping
                // darts/vertices; This would indicate that the face is incorrectly built
                let seg_normal = seg_dir.normal_dir().unwrap();
                let vr = center + seg_dir * T::from(0.01).unwrap();
                let vl = center - seg_dir * T::from(0.01).unwrap();
                let vt = center + seg_normal * T::from(0.1).unwrap();
                let vb = center - seg_normal * T::from(0.1).unwrap();
                [
                    Coords2Shader::from((vt, Entity::Beta)),
                    Coords2Shader::from((vl, Entity::Beta)),
                    Coords2Shader::from((vr, Entity::Beta)),
                    Coords2Shader::from((vl, Entity::Beta)),
                    Coords2Shader::from((vr, Entity::Beta)),
                    Coords2Shader::from((vb, Entity::Beta)),
                ]
                .into_iter()
            });
        self._beta_construction_buffer.extend(tmp);
    }

    pub fn build_faces(&mut self) {
        // because there's no "trianglefan" primitive in the webgpu standard,
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
        self.vertices.append(&mut self.face_construction_buffer);
        self.vertices.append(&mut self.dart_construction_buffer);
        self.vertices.append(&mut self._beta_construction_buffer);
    }

    pub fn vertices(&self) -> &[Coords2Shader] {
        &self.vertices
    }
}
