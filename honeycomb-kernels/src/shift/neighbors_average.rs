use honeycomb_core::{
    cmap::{CMap2, DartIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID},
    fast_stm::atomically,
    prelude::{CoordsFloat, Vertex2},
};
use rayon::prelude::*;

pub fn shift<T: CoordsFloat>(cmap: &CMap2<T>, n_rounds: usize) {
    // fetch all vertices that are not on the boundary of the map
    let vertices: Vec<VertexIdType> = cmap
        .fetch_vertices()
        .identifiers
        .into_par_iter()
        .filter(|v| {
            !Orbit2::new(&cmap, OrbitPolicy::Vertex, *v as DartIdType)
                .any(|d| cmap.beta::<2>(d) == NULL_DART_ID)
        })
        .collect();

    let neighbors: Vec<Vec<VertexIdType>> = vertices
        .par_iter()
        .map(|v| {
            Orbit2::new(&cmap, OrbitPolicy::Vertex, *v as DartIdType)
                .map(|d| cmap.vertex_id(cmap.beta::<2>(d)))
                .collect()
        })
        .collect();

    // main loop
    let mut round = 0;
    loop {
        vertices
            .iter()
            .zip(neighbors.iter())
            .par_bridge()
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
        round += 1;
        if round >= n_rounds {
            break;
        }
    }
}
