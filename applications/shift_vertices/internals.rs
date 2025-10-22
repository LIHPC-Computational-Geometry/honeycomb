use honeycomb::{
    prelude::{
        CMap2, CoordsFloat, DartIdType, NULL_DART_ID, OrbitPolicy, VertexIdType,
        remeshing::move_vertex_to_average,
    },
    stm::atomically,
};
use rayon::prelude::*;

pub fn build_vertex_graph<T: CoordsFloat>(
    map: &CMap2<T>,
    sort: bool,
) -> Vec<(VertexIdType, Vec<VertexIdType>)> {
    if sort {
        todo!("currently unimplemented");
    } else {
        let instant = std::time::Instant::now();
        let tmp = map
            .par_iter_vertices()
            .filter_map(|v| {
                if map
                    .orbit(OrbitPolicy::Vertex, v as DartIdType)
                    .any(|d| map.beta::<2>(d) == NULL_DART_ID)
                {
                    None
                } else {
                    Some((
                        v,
                        map.orbit(OrbitPolicy::Vertex, v as DartIdType)
                            .map(|d| map.vertex_id(map.beta::<2>(d)))
                            .collect(),
                    ))
                }
            })
            .collect();
        println!("| |-> graph built in {}ms", instant.elapsed().as_millis());

        tmp
    }
}

pub fn shift<T: CoordsFloat>(
    map: &CMap2<T>,
    graph: &[(VertexIdType, Vec<VertexIdType>)],
    n_rounds: usize,
) {
    println!(" Round | process_time | throughput(vertex/s)");
    // main loop
    let mut round = 0;
    let mut process_time;
    let n_v = graph.len();
    loop {
        let instant = std::time::Instant::now();
        graph.par_iter().for_each(|(vid, neigh)| {
            atomically(|t| move_vertex_to_average(t, map, *vid, neigh));
        });
        process_time = instant.elapsed().as_secs_f64();
        println!(
            " {:>5} | {:>12.6e} | {:>20.6e}",
            round,
            process_time,
            n_v as f64 / process_time,
        );

        round += 1;
        if round >= n_rounds {
            break;
        }
    }
}
