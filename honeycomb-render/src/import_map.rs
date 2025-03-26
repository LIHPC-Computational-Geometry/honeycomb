use bevy::prelude::{Bundle, Commands, Component, Res, Resource, Vec3};
use bevy::utils::HashMap;
use honeycomb_core::cmap::NULL_DART_ID;
use honeycomb_core::{
    cmap::{
        CMap2, CMap3, DartIdType, EdgeIdType, FaceIdType, OrbitPolicy, VertexIdType, VolumeIdType,
    },
    geometry::CoordsFloat,
};
use itertools::Itertools;

// --- shared data

/// Combinatorial map to render.
#[derive(Resource)]
pub struct Map<T: CoordsFloat>(pub CMap2<T>);

/// Combinatorial map to render.
#[derive(Resource)]
pub struct Map3<T: CoordsFloat>(pub CMap3<T>);

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
    pub(crate) volume_id: VolumeId,
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

/// Build ECS data from a combinatorial map object.
///
/// # Panics
///
/// This function will panic if there is a topological vertex with no associated coordinates.
#[allow(clippy::too_many_lines)]
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
                // a first normalization is required because both edges should weight equally
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
                    volume_id: VolumeId(1),
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

    for bundle in darts {
        commands.spawn(bundle);
    }
    for bundle in vertices {
        commands.spawn(bundle);
    }
    for bundle in edges {
        commands.spawn(bundle);
    }
    for bundle in faces {
        commands.spawn(bundle);
    }
}

/// Build ECS data from a combinatorial map object.
///
/// # Panics
///
/// This function will panic if there is a topological vertex with no associated coordinates.
#[allow(clippy::too_many_lines)]
pub fn extract_data_from_3d_map<T: CoordsFloat>(mut commands: Commands, cmap: Res<Map3<T>>) {
    let cmap = &cmap.0;
    let map_vertices: Vec<_> = cmap.iter_vertices().collect();
    println!("nv: {}", map_vertices.len());
    let map_edges: Vec<_> = cmap.iter_edges().collect();
    let map_faces: Vec<_> = cmap.iter_faces().collect();
    let map_volumes: Vec<_> = cmap.iter_volumes().collect();

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
            Vec3::from((
                v.0.to_f32().unwrap(),
                v.1.to_f32().unwrap(),
                v.2.to_f32().unwrap(),
            ))
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
            let v2id = if cmap.is_i_free::<3>(*id as DartIdType) {
                if cmap.is_i_free::<2>(*id as DartIdType) {
                    cmap.vertex_id(cmap.beta::<1>(*id as DartIdType))
                } else {
                    cmap.vertex_id(cmap.beta::<2>(*id as DartIdType))
                }
            } else {
                cmap.vertex_id(cmap.beta::<3>(*id as DartIdType))
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
                .orbit(OrbitPolicy::Custom(&[1]), *id as DartIdType) // cannot use FaceLinear here due to doubled darts in 3D
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
                // a first normalization is required because both edges should weight equally
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
                    volume_id: VolumeId(cmap.volume_id(*dart_id)),
                    dart: Dart {
                        start: *v1_id,
                        end: *v2_id,
                    },
                }
            });

            darts.extend(bodies);

            // common dart iterator
            let mut tmp2 = cmap
                .orbit(OrbitPolicy::Custom(&[1]), cmap.beta::<3>(*id as DartIdType))
                .filter_map(|dart_id| {
                    if dart_id != NULL_DART_ID {
                        Some((dart_id, index_map[&cmap.vertex_id(dart_id)]))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if !tmp2.is_empty() {
                tmp2.push(tmp2[0]); // trick for the `.windows` call
            }
            // dart bodies
            let bodies2 = tmp2.windows(2).map(|wd| {
                let [(dart_id, v1_id), (_, v2_id)] = wd else {
                    unreachable!("i kneel");
                };

                DartBundle {
                    id: DartId(*dart_id),
                    vertex_id: VertexId(cmap.vertex_id(*dart_id)),
                    edge_id: EdgeId(cmap.edge_id(*dart_id)),
                    face_id: FaceId(*id),
                    volume_id: VolumeId(cmap.volume_id(*dart_id)),
                    dart: Dart {
                        start: *v1_id,
                        end: *v2_id,
                    },
                }
            });

            darts.extend(bodies2);

            FaceBundle {
                id: FaceId(*id),
                face: Face(vertex_ids),
            }
        })
        .collect();

    let mut volume_normals = HashMap::new();

    map_volumes.iter().for_each(|vol| {
        let darts: Vec<_> = cmap
            .orbit(OrbitPolicy::Volume, *vol as DartIdType)
            .collect();
        let norms: HashMap<_, _> = darts
            .iter()
            .unique_by(|&d| cmap.face_id(*d))
            .map(|d| {
                let mut base = Vec3::default();
                let tmp: Vec<_> = cmap
                    .orbit(OrbitPolicy::Custom(&[1]), *d as DartIdType)
                    .chain([*d].into_iter())
                    .collect();
                tmp.windows(2).for_each(|sl| {
                    let [d1, d2] = sl else { unreachable!() };
                    let (vid1, vid2) = (
                        index_map[&cmap.vertex_id(*d1)],
                        index_map[&cmap.vertex_id(*d2)],
                    );
                    let (v1, v2) = (vertex_vals[vid1], vertex_vals[vid2]);
                    base.x += (v1.y - v2.y) * (v1.z + v2.z);
                    base.y += (v1.z - v2.z) * (v1.x + v2.x);
                    base.z += (v1.x - v2.x) * (v1.y + v2.y);
                });
                (cmap.face_id(*d), base.normalize())
            })
            .collect();

        // maps are cool => in the vertex/volume suborbit, there is exactly a single dart belonging
        // to each intersecting faces.
        darts.iter().for_each(|d| {
            let fid = cmap.face_id(*d);
            let vid = index_map[&cmap.vertex_id(*d)];
            if let Some(val) = volume_normals.get_mut(&(*vol, vid)) {
                *val += norms[&fid];
            } else {
                volume_normals.insert((*vol, vid), norms[&fid]);
            }
        });
    });
    for val in volume_normals.values_mut() {
        *val = val.normalize();
    }

    commands.insert_resource(MapVertices(vertex_vals));
    commands.insert_resource(FaceNormals(face_normals));
    commands.insert_resource(VolumeNormals(volume_normals));

    for bundle in darts {
        commands.spawn(bundle);
    }
    for bundle in vertices {
        commands.spawn(bundle);
    }
    for bundle in edges {
        commands.spawn(bundle);
    }
    for bundle in faces {
        commands.spawn(bundle);
    }
}
