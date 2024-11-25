use honeycomb_core::{
    cmap::{CMap2, DartIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID},
    prelude::{CoordsFloat, Vertex2},
    stm::atomically,
};

pub fn shift<T: CoordsFloat>(cmap: &CMap2<T>) {
    // fetch all vertices that are not on the boundary of the map
    let vertices: Vec<VertexIdType> = cmap
        .fetch_vertices()
        .identifiers
        .into_iter()
        .filter(|v| {
            !Orbit2::new(&cmap, OrbitPolicy::Vertex, *v as DartIdType)
                .any(|d| cmap.beta::<2>(d) == NULL_DART_ID)
        })
        .collect();

    let neighbors: Vec<Vec<VertexIdType>> = vertices
        .iter()
        .map(|v| {
            Orbit2::new(&cmap, OrbitPolicy::Vertex, *v as DartIdType)
                .map(|d| cmap.vertex_id(cmap.beta::<2>(d)))
                .collect()
        })
        .collect();

    // main loop
    vertices
        .iter()
        .zip(neighbors.iter())
        .for_each(|(vid, neigh)| {
            atomically(|trans| {
                let mut new_val: Vertex2<T> = Vertex2::default();
                for v in neigh {
                    let vertex = cmap.read_vertex(trans, *v)?.unwrap();
                    new_val.0 += vertex.0;
                    new_val.1 += vertex.1;
                }
                new_val.0 /= T::from(neigh.len()).unwrap();
                new_val.1 /= T::from(neigh.len()).unwrap();
                cmap.write_vertex(trans, *vid, new_val)
            });
        });
}
