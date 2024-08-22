pub mod ecs_data;
mod system;

use crate::capture::ecs_data::CaptureId;
use crate::capture::system::{populate_darts, populate_edges, populate_faces, populate_vertices};
use crate::{DartBodyBundle, DartHeadBundle, EdgeBundle, FaceBundle, VertexBundle};
use bevy::prelude::*;
use bevy::utils::HashMap;
use honeycomb_core::{
    CMap2, CoordsFloat, DartIdentifier, FaceIdentifier, Orbit2, OrbitPolicy, VertexIdentifier,
};

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        // resource
        app.insert_resource(FocusedCapture::default())
            .insert_resource(CaptureList::default());
        // systems
        app.add_systems(Startup, populate_darts)
            .add_systems(Startup, populate_vertices)
            .add_systems(Startup, populate_edges)
            .add_systems(Startup, populate_faces);
    }
}

#[derive(Resource)]
pub struct FocusedCapture(pub CaptureId);

impl Default for FocusedCapture {
    fn default() -> Self {
        Self(CaptureId(0))
    }
}

#[derive(Resource)]
pub struct CaptureList(pub Vec<Capture>);

impl Default for CaptureList {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub struct Capture {
    pub metadata: CaptureMD,
    pub vertex_vals: Vec<Vec3>,
    pub normals: HashMap<FaceIdentifier, Vec<Vec3>>,
    pub darts: Vec<(DartHeadBundle, DartBodyBundle)>,
    pub vertices: Vec<VertexBundle>,
    pub edges: Vec<EdgeBundle>,
    pub faces: Vec<FaceBundle>,
}

impl Capture {
    pub fn new<T: CoordsFloat>(cap_id: usize, cmap: &CMap2<T>) -> Result<Capture, String> {
        let map_vertices = cmap.fetch_vertices();
        let map_edges = cmap.fetch_edges();
        let map_faces = cmap.fetch_faces();
        let metadata = CaptureMD {
            capture_id: cap_id,
            n_darts: cmap.n_darts() - cmap.n_unused_darts(),
            n_vertices: cmap.n_vertices(),
            n_edges: map_edges.identifiers.len(),
            n_faces: map_faces.identifiers.len(),
            n_volumes: 0,
        };

        let mut index_map: HashMap<VertexIdentifier, usize> =
            HashMap::with_capacity(cmap.n_vertices());

        let vertex_vals: Vec<Vec3> = map_vertices
            .identifiers
            .iter()
            .enumerate()
            .map(|(idx, vid)| {
                index_map.insert(*vid, idx);
                let v = cmap.vertex(*vid).unwrap();
                Vec3::from((v.0.to_f32().unwrap(), v.1.to_f32().unwrap(), 0.0))
            })
            .collect();

        let vertices: Vec<VertexBundle> = map_vertices
            .identifiers
            .iter()
            .map(|id| VertexBundle::new(cap_id, *id, index_map[id]))
            .collect();

        let edges: Vec<EdgeBundle> = map_edges
            .identifiers
            .iter()
            .map(|id| {
                let v1id = cmap.vertex_id(*id as DartIdentifier);
                let v2id = if cmap.is_i_free::<2>(*id as DartIdentifier) {
                    cmap.vertex_id(cmap.beta::<1>(*id as DartIdentifier))
                } else {
                    cmap.vertex_id(cmap.beta::<2>(*id as DartIdentifier))
                };
                EdgeBundle::new(cap_id, *id, (index_map[&v1id], index_map[&v2id]))
            })
            .collect();

        let mut normals = HashMap::new();
        let mut darts: Vec<(DartHeadBundle, DartBodyBundle)> = Vec::new();

        let faces: Vec<FaceBundle> = map_faces
            .identifiers
            .iter()
            .map(|id| {
                let vertex_ids: Vec<usize> =
                    Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), *id as DartIdentifier)
                        .map(|dart_id| index_map[&cmap.vertex_id(dart_id)])
                        .collect();
                let n_v = vertex_ids.len();
                let mut loc_normals = vec![{
                    let (ver_in, ver, ver_out) =
                        (&vertex_ids[n_v - 1], &vertex_ids[0], &vertex_ids[1]);
                    let (vec_in, vec_out) = (
                        vertex_vals[*ver] - vertex_vals[*ver_in],
                        vertex_vals[*ver_out] - vertex_vals[*ver],
                    );
                    // vec_in/out belong to X/Y plane => .cross(Z) == normal in the plane
                    (vec_in.cross(Vec3::Z) + vec_out.cross(Vec3::Z)).normalize()
                }];
                loc_normals.extend(vertex_ids.windows(3).map(|wdw| {
                    let [ver_in, ver, ver_out] = wdw else {
                        unreachable!("i kneel");
                    };
                    let (vec_in, vec_out) = (
                        vertex_vals[*ver] - vertex_vals[*ver_in],
                        vertex_vals[*ver_out] - vertex_vals[*ver],
                    );
                    (vec_in.cross(Vec3::Z) + vec_out.cross(Vec3::Z)).normalize()
                }));
                loc_normals.push({
                    let (ver_in, ver, ver_out) =
                        (&vertex_ids[n_v - 2], &vertex_ids[n_v - 1], &vertex_ids[0]);
                    let (vec_in, vec_out) = (
                        vertex_vals[*ver] - vertex_vals[*ver_in],
                        vertex_vals[*ver_out] - vertex_vals[*ver],
                    );
                    (vec_in.cross(Vec3::Z) + vec_out.cross(Vec3::Z)).normalize()
                });

                normals.insert(*id, loc_normals);

                // common dart iterator
                let mut tmp = Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), *id as DartIdentifier)
                    .zip(vertex_ids.iter().enumerate())
                    .map(|(dart_id, (vertex_id, loc_normal_id))| {
                        (dart_id, vertex_id, *loc_normal_id)
                    })
                    .collect::<Vec<_>>();
                tmp.push(tmp[0]); // trick for the `.windows` call

                // dart bodies
                let bodies = tmp.windows(2).map(|wd| {
                    let [(dart_id, v1_id, v1n_id), (_, v2_id, v2n_id)] = wd else {
                        unreachable!("i kneel");
                    };
                    DartBodyBundle::new(
                        cap_id,
                        *dart_id,
                        cmap.vertex_id(*dart_id),
                        cmap.edge_id(*dart_id),
                        *id,
                        (*v1_id, *v2_id),
                        (*v1n_id, *v2n_id),
                    )
                });

                let heads =
                    tmp[0..tmp.len() - 2]
                        .iter()
                        .map(|(dart_id, vertex_id, loc_normal_id)| {
                            DartHeadBundle::new(
                                cap_id,
                                *dart_id,
                                cmap.vertex_id(*dart_id),
                                cmap.edge_id(*dart_id),
                                *id,
                                *vertex_id,
                                *loc_normal_id,
                            )
                        });

                darts.extend(heads.zip(bodies));

                FaceBundle::new(cap_id, *id, vertex_ids)
            })
            .collect();

        Ok(Self {
            metadata,
            vertex_vals,
            normals,
            darts,
            vertices,
            edges,
            faces,
        })
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
