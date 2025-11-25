use honeycomb::{
    prelude::{
        CMap2, CoordsFloat, DartIdType, NULL_DART_ID, OrbitPolicy, VertexIdType,
        remeshing::{move_vertex_to_average, neighbor_based_smooth},
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

pub fn laplace<T: CoordsFloat>(
    map: &CMap2<T>,
    graph: &[(VertexIdType, Vec<VertexIdType>)],
    n_rounds: usize,
    lambda: T,
) {
    println!(" Round | process_time | throughput(vertex/s)");
    // main loop
    let mut round = 0;
    let mut process_time;
    let n_v = graph.len();
    loop {
        let instant = std::time::Instant::now();
        graph.par_iter().for_each(|(vid, neigh)| {
            atomically(|t| neighbor_based_smooth(t, map, *vid, neigh, lambda));
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

pub fn taubin<T: CoordsFloat>(
    map: &CMap2<T>,
    graph: &[(VertexIdType, Vec<VertexIdType>)],
    n_rounds: usize,
    lambda: T,
    k: T,
) {
    println!(" Round | process_time | throughput(vertex/s)");
    // main loop
    let mut round = 0;
    let mut process_time;
    let n_v = graph.len();

    let mu = T::one() / (k - T::one() / lambda);

    loop {
        let instant = std::time::Instant::now();
        let scale = if round % 2 == 0 { lambda } else { mu };
        graph.par_iter().for_each(|(vid, neigh)| {
            atomically(|t| neighbor_based_smooth(t, map, *vid, neigh, scale));
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
