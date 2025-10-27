use honeycomb_core::{
    cmap::{CMap2, DartIdType, OrbitPolicy, VertexIdType, NULL_DART_ID},
    geometry::Vertex2,
    stm::atomically,
};
use honeycomb_kernels::grid_generation::GridBuilder;
use rayon::prelude::*;

const N_SQUARES: usize = 256;
const N_ROUNDS: usize = 100;

fn main() {
    let map: CMap2<f64> = GridBuilder::<2, _>::unit_triangles(N_SQUARES);

    // fetch all vertices that are not on the boundary of the map
    let nodes: Vec<(VertexIdType, Vec<VertexIdType>)> = map
        .iter_vertices()
        .filter_map(|v| {
            // the condition detects if we're on the boundary
            if map
                .orbit(OrbitPolicy::Vertex, v as DartIdType)
                .any(|d| map.beta::<2>(d) == NULL_DART_ID)
            {
                None
            } else {
                // the orbit transformation yields neighbor IDs
                Some((
                    v,
                    map.orbit(OrbitPolicy::Vertex, v as DartIdType)
                        .map(|d| map.vertex_id(map.beta::<2>(d)))
                        .collect(),
                ))
            }
        })
        .collect();

    // main loop
    let mut round = 0;
    loop {
        // process nodes in parallel
        nodes.par_iter().for_each(|(vid, neigh)| {
            // we need a transaction here to avoid UBs, since there's
            // no guarantee we won't process neighbor nodes concurrently
            //
            // the transaction will ensure that we do not validate an operation
            // where inputs have changed due to instruction interleaving between threads
            // here, it will retry the transaction until it can be validated
            atomically(|trans| {
                let mut new_val = Vertex2::default();
                for v in neigh {
                    let vertex = map.read_vertex(trans, *v)?.unwrap();
                    new_val.0 += vertex.0;
                    new_val.1 += vertex.1;
                }
                new_val.0 /= neigh.len() as f64;
                new_val.1 /= neigh.len() as f64;
                map.write_vertex(trans, *vid, new_val)
            });
        });

        round += 1;
        if round >= N_ROUNDS {
            break;
        }
    }

    std::hint::black_box(map);
}
