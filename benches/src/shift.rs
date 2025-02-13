//! # Description
//!
//! ## Routine
//!
//! The algorithm fetches all vertices that are not on the border of the map, fetch all identifiers
//! of each respective vertices' neighbors. Then, for all vertices:
//!
//! - compute the average between neighbors
//! - overwrite current vertex value with computed average
//!
//! ## Benchmark
//!
//! This binary is meant to be use to evaluate scalability of geometry-only kernels. It is
//! parallelized using rayon, and the number of thread used for execution can be controlled using
//! `taskset`. By controlling this, and the grid size, we can evaluate both strong and weak
//! scaling characteristics.

use rayon::prelude::*;

use honeycomb::core::stm::atomically;
use honeycomb::prelude::{
    CMap2, CMapBuilder, CoordsFloat, DartIdType, Orbit2, OrbitPolicy, Vertex2, VertexIdType,
    NULL_DART_ID,
};

use crate::cli::ShiftArgs;

pub fn bench_shift<T: CoordsFloat>(args: ShiftArgs) -> CMap2<T> {
    let map: CMap2<T> = CMapBuilder::from(args.input).build().unwrap();

    if args.no_conflict {
        todo!("TODO: require a partitioning algorithm")
    } else {
        // fetch all vertices that are not on the boundary of the map
        let tmp: Vec<(VertexIdType, Vec<VertexIdType>)> = map
            .iter_vertices()
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
            })
            .collect();
        // main loop
        let mut round = 0;
        loop {
            tmp.par_iter().for_each(|(vid, neigh)| {
                atomically(|trans| {
                    let mut new_val = Vertex2::default();
                    for v in neigh {
                        let vertex = map.read_vertex(trans, *v)?.unwrap();
                        new_val.0 += vertex.0;
                        new_val.1 += vertex.1;
                    }
                    new_val.0 /= T::from(neigh.len()).unwrap();
                    new_val.1 /= T::from(neigh.len()).unwrap();
                    map.write_vertex(trans, *vid, new_val)
                });
            });

            round += 1;
            if round >= args.n_rounds {
                break;
            }
        }
    }

    map
}
