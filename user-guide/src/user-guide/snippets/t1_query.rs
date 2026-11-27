use honeycomb::prelude::*;

pub fn main() {
    let map: CMap2<f64> = grid_generation::GridBuilder::<2, f64>::unit_triangles(4);

    // iterate over cells
    let n_vertices = map.iter_vertices().count();
    let edges: Vec<EdgeIdType> = map.iter_edges().collect();
    let faces: Vec<FaceIdType> = map.iter_faces().collect();

    // fetch cells associated to a given dart
    let dart: DartIdType = 1;
    let vertex_of_dart = map.vertex_id(dart);
    let edge_of_dart = map.edge_id(dart);
    let face_of_dart = map.face_id(dart);

    // fetch all the darts of a given cell
    let vertex_darts: Vec<DartIdType> = map.orbit(OrbitPolicy::Vertex, dart).collect();
    let face_darts: Vec<DartIdType> = map.i_cell::<2>(dart).collect();

    // direct vertex processing
    map.iter_vertices().for_each(|vid| {
        if let Some(pos) = map.read_vertex(vid) {
            println!("vertex {vid} is at ({}, {})", pos.x(), pos.y());
        } else {
            println!("vertex {vid} has no associated coordinates");
        }
    });
}
