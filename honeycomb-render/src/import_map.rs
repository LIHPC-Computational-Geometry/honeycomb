use bevy::prelude::*;
use bevy::utils::HashMap;
use honeycomb_core::{
    cmap::{CMap2, DartIdType, EdgeIdType, FaceIdType, OrbitPolicy, VertexIdType, VolumeIdType},
    geometry::CoordsFloat,
};

// --- shared data

#[derive(Resource)]
pub struct Map<T: CoordsFloat>(pub CMap2<T>);

/// Collection of a map's vertices.
#[derive(Resource)]
pub struct MapVertices(pub Vec<Vec3>);

/// Collection of normals, organized per faces of a map.
#[derive(Resource)]
pub struct FaceNormals(pub HashMap<(FaceIdType, usize), Vec3>);

/// Collection of normals, organized per volumes of a map.
#[derive(Resource)]
pub struct VolumeNormals(pub HashMap<(VolumeIdType, usize), Vec3>);

// --- bundles

/// Bundle used to create entities modeling dart bodies.
#[derive(Bundle, Clone)]
pub struct DartBundle {
    pub(crate) id: DartId,
    pub(crate) vertex_id: VertexId,
    pub(crate) edge_id: EdgeId,
    pub(crate) face_id: FaceId,
    pub(crate) dart: Dart,
}

/// Bundle used to create entities modeling vertices.
#[derive(Bundle, Clone)]
pub struct VertexBundle {
    pub(crate) id: VertexId,
    pub(crate) vertex: Vertex,
}

/// Bundle used to create entities modeling edges.
#[derive(Bundle, Clone)]
pub struct EdgeBundle {
    pub(crate) id: EdgeId,
    pub(crate) edge: Edge,
}

/// Bundle used to create entities modeling faces.
#[derive(Bundle, Clone)]
pub struct FaceBundle {
    pub(crate) id: FaceId,
    pub(crate) face: Face,
}

// --- individual components

/// Dart ID component.
#[derive(Component, Clone)]
pub struct DartId(pub DartIdType);

/// Vertex ID component.
#[derive(Component, Clone)]
pub struct VertexId(pub VertexIdType);

/// Edge ID component.
#[derive(Component, Clone)]
pub struct EdgeId(pub EdgeIdType);

/// Face ID component.
#[derive(Component, Clone)]
pub struct FaceId(pub FaceIdType);

/// Volume ID component.
#[derive(Component, Clone)]
pub struct VolumeId(pub VolumeIdType);

/// Dart head component.
#[derive(Component, Clone)]
pub struct Dart {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// Beta component.
#[derive(Component, Clone)]
pub struct Beta(pub u8, pub usize, pub usize); // beta id, v0_id, v1_id ?

/// Vertex component.
#[derive(Component, Clone)]
pub struct Vertex(pub usize); // map id, vid

/// Edge component.
#[derive(Component, Clone)]
pub struct Edge(pub usize, pub usize); // v0_id, v1_id

/// Face component.
#[derive(Component, Clone)]
pub struct Face(pub Vec<usize>); // vertex list

/// Volume component.
#[derive(Component, Clone)]
pub struct Volume;

// --- startup routine

pub fn extract_data_from_map<T: CoordsFloat>(mut commands: Commands, cmap: Res<Map<T>>) {
    let cmap = &cmap.0;
    let map_vertices: Vec<_> = cmap.iter_vertices().collect();
    let map_edges: Vec<_> = cmap.iter_edges().collect();
    let map_faces: Vec<_> = cmap.iter_faces().collect();

    let mut index_map: HashMap<VertexIdType, usize> = HashMap::with_capacity(cmap.n_vertices());

    let vertex_vals: Vec<Vec3> = map_vertices
        .iter()
        .enumerate()
        .map(|(idx, vid)| {
            index_map.insert(*vid, idx);
            let v = cmap
                .force_read_vertex(*vid)
                .expect("E: found a topological vertex with no associated coordinates");
            // sane unwraps; will crash if the coordinates cannot be converted to f32
            Vec3::from((v.0.to_f32().unwrap(), v.1.to_f32().unwrap(), 0.0))
        })
        .collect();

    let vertices: Vec<VertexBundle> = map_vertices
        .iter()
        .map(|id| VertexBundle {
            id: VertexId(*id),
            vertex: Vertex(index_map[id]),
        })
        .collect();

    let edges: Vec<EdgeBundle> = map_edges
        .iter()
        .map(|id| {
            let v1id = cmap.vertex_id(*id as DartIdType);
            let v2id = if cmap.is_i_free::<2>(*id as DartIdType) {
                cmap.vertex_id(cmap.beta::<1>(*id as DartIdType))
            } else {
                cmap.vertex_id(cmap.beta::<2>(*id as DartIdType))
            };
            EdgeBundle {
                id: EdgeId(*id),
                edge: Edge(index_map[&v1id], index_map[&v2id]),
            }
        })
        .collect();

    let mut face_normals = HashMap::new();
    let mut darts: Vec<DartBundle> = Vec::new();

    let faces: Vec<FaceBundle> = map_faces
        .iter()
        .map(|id| {
            let vertex_ids: Vec<usize> = cmap
                .orbit(OrbitPolicy::Custom(&[1]), *id as DartIdType)
                .map(|dart_id| index_map[&cmap.vertex_id(dart_id)])
                .collect();
            let n_v = vertex_ids.len();
            {
                let (ver_in, ver, ver_out) = (&vertex_ids[n_v - 1], &vertex_ids[0], &vertex_ids[1]);
                let (vec_in, vec_out) = (
                    vertex_vals[*ver] - vertex_vals[*ver_in],
                    vertex_vals[*ver_out] - vertex_vals[*ver],
                );
                // vec_in/out belong to X/Y plane => .cross(Z) == normal in the plane
                // a firts normalization is required because both edges should weight equally
                let normal = (vec_in.cross(Vec3::Z).normalize()
                    + vec_out.cross(Vec3::Z).normalize())
                .normalize();
                face_normals.insert((*id, *ver), normal);
            }
            vertex_ids.windows(3).for_each(|wdw| {
                let [ver_in, ver, ver_out] = wdw else {
                    unreachable!("i kneel");
                };
                let (vec_in, vec_out) = (
                    vertex_vals[*ver] - vertex_vals[*ver_in],
                    vertex_vals[*ver_out] - vertex_vals[*ver],
                );
                let normal = (vec_in.cross(Vec3::Z).normalize()
                    + vec_out.cross(Vec3::Z).normalize())
                .normalize();
                face_normals.insert((*id, *ver), normal);
            });
            {
                let (ver_in, ver, ver_out) =
                    (&vertex_ids[n_v - 2], &vertex_ids[n_v - 1], &vertex_ids[0]);
                let (vec_in, vec_out) = (
                    vertex_vals[*ver] - vertex_vals[*ver_in],
                    vertex_vals[*ver_out] - vertex_vals[*ver],
                );
                let normal = (vec_in.cross(Vec3::Z).normalize()
                    + vec_out.cross(Vec3::Z).normalize())
                .normalize();
                face_normals.insert((*id, *ver), normal);
            }

            // common dart iterator
            let mut tmp = cmap
                .orbit(OrbitPolicy::Custom(&[1]), *id as DartIdType)
                .map(|dart_id| (dart_id, index_map[&cmap.vertex_id(dart_id)]))
                .collect::<Vec<_>>();
            tmp.push(tmp[0]); // trick for the `.windows` call

            // dart bodies
            let bodies = tmp.windows(2).map(|wd| {
                let [(dart_id, v1_id), (_, v2_id)] = wd else {
                    unreachable!("i kneel");
                };

                DartBundle {
                    id: DartId(*dart_id),
                    vertex_id: VertexId(cmap.vertex_id(*dart_id)),
                    edge_id: EdgeId(cmap.edge_id(*dart_id)),
                    face_id: FaceId(*id),
                    dart: Dart {
                        start: *v1_id,
                        end: *v2_id,
                    },
                }
            });

            darts.extend(bodies);

            FaceBundle {
                id: FaceId(*id),
                face: Face(vertex_ids),
            }
        })
        .collect();

    commands.insert_resource(MapVertices(vertex_vals));
    commands.insert_resource(FaceNormals(face_normals));

    vertices.into_iter().for_each(|bundle| {
        commands.spawn(bundle);
    });
    edges.into_iter().for_each(|bundle| {
        commands.spawn(bundle);
    });
    faces.into_iter().for_each(|bundle| {
        commands.spawn(bundle);
    });
}
