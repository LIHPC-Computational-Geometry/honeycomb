use honeycomb::prelude::*;

pub fn smooth_vertex<T: CoordsFloat>(map: &CMap2<T>, vid: VertexIdType) {
    // neighbors of the vertex: follow beta_2 around the vertex orbit
    let neighbors: Vec<VertexIdType> = map
        .orbit(OrbitPolicy::Vertex, vid as DartIdType)
        .map(|d| map.vertex_id(map.beta::<2>(d)))
        .collect();

    // average of the neighbors' positions
    let mut new_pos = Vertex2::default();
    for n in &neighbors {
        let v = map
            .read_vertex(*n)
            .expect("E: no associated coordinates to vertex");
        new_pos.0 += v.0;
        new_pos.1 += v.1;
    }
    let n = T::from(neighbors.len()).unwrap();
    new_pos.0 /= n;
    new_pos.1 /= n;

    map.write_vertex(vid, new_pos);
}
