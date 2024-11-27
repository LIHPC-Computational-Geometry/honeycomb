use rayon::prelude::*;

use honeycomb::core::stm::atomically;
use honeycomb::prelude::{
    CMap2, CMapBuilder, DartIdType, Orbit2, OrbitPolicy, Vertex2, VertexIdType, NULL_DART_ID,
};

fn main() {
    // ./binary grid_size n_rounds
    let args: Vec<String> = std::env::args().collect();
    let n_squares = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256);
    let n_rounds = args
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    let map: CMap2<f64> = CMapBuilder::unit_grid(n_squares).build().unwrap();

    // fetch all vertices that are not on the boundary of the map
    let tmp = map
        .fetch_vertices()
        .identifiers
        .into_iter()
        .filter_map(|v| {
            if Orbit2::new(&map, OrbitPolicy::Vertex, v as DartIdType)
                .any(|d| map.beta::<2>(d) == NULL_DART_ID)
            {
                None
            } else {
                Some((
                    v,
                    Orbit2::new(&map, OrbitPolicy::Vertex, v as DartIdType)
                        .map(|d| map.vertex_id(map.beta::<2>(d)))
                        .collect(),
                ))
            }
        });

    #[allow(clippy::type_complexity)]
    let (first_batch, second_batch): (
        Vec<(VertexIdType, Vec<VertexIdType>)>,
        Vec<(VertexIdType, Vec<VertexIdType>)>,
    ) = tmp.partition(|(v, _)| ((v - 1) / 4) % 2 == 0); // this yields 2 ind. batches, just trust me

    // main loop
    let mut round = 0;
    loop {
        first_batch.par_iter().for_each(|(vid, neigh)| {
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
        second_batch.par_iter().for_each(|(vid, neigh)| {
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
        if round >= n_rounds {
            break;
        }
    }

    std::hint::black_box(map);
}
