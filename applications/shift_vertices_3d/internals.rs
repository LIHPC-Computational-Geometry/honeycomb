use honeycomb::{
    prelude::{
        CMap3, CoordsFloat, DartIdType, NULL_DART_ID, OrbitPolicy, VertexIdType,
        remeshing::{move_vertex_to_average_3d, neighbor_based_smooth_3d},
    },
    stm::atomically,
};
use rayon::prelude::*;
use rustc_hash::FxHashSet as HashSet;

fn vertex_neighbors<T: CoordsFloat>(map: &CMap3<T>, v: VertexIdType) -> Vec<VertexIdType> {
    let mut neighbors = HashSet::default();
    for dart in map.orbit(OrbitPolicy::Vertex, v as DartIdType) {
        let beta_2 = map.beta::<2>(dart);
        let neighbor = if beta_2 == NULL_DART_ID {
            map.vertex_id(map.beta::<1>(dart))
        } else {
            map.vertex_id(beta_2)
        };
        if neighbor != v && neighbor != NULL_DART_ID {
            neighbors.insert(neighbor);
        }

        let beta_0 = map.beta::<0>(dart);
        if beta_0 != NULL_DART_ID && map.beta::<2>(beta_0) == NULL_DART_ID {
            let neighbor = map.vertex_id(beta_0);
            if neighbor != v {
                neighbors.insert(neighbor);
            }
        }
    }
    neighbors.into_iter().collect()
}

pub fn build_vertex_graph<T: CoordsFloat>(
    map: &CMap3<T>,
    sort: bool,
) -> Vec<(VertexIdType, Vec<VertexIdType>)> {
    if sort {
        todo!("currently unimplemented");
    } else {
        let instant = std::time::Instant::now();
        let tmp = map
            .par_iter_vertices()
            .map(|v| (v, vertex_neighbors(map, v)))
            .collect();
        println!("| |-> graph built in {}ms", instant.elapsed().as_millis());

        tmp
    }
}

#[cfg(test)]
mod tests {
    use honeycomb::prelude::grid_generation::GridBuilder;

    use super::build_vertex_graph;

    #[test]
    fn boundary_vertices_have_complete_neighborhoods() {
        let map = GridBuilder::<3, f64>::hex_grid(1, 1.0);
        let graph = build_vertex_graph(&map, false);

        assert_eq!(graph.len(), 8);
        assert!(graph.iter().all(|(_, neighbors)| neighbors.len() == 3));
    }
}

pub fn shift<T: CoordsFloat>(
    map: &CMap3<T>,
    graph: &[(VertexIdType, Vec<VertexIdType>)],
    n_rounds: usize,
) {
    println!(" Round | process_time | throughput(vertex/s)");
    let mut round = 0;
    let n_v = graph.len();
    loop {
        let instant = std::time::Instant::now();
        graph.par_iter().for_each(|(vid, neigh)| {
            atomically(|t| move_vertex_to_average_3d(t, map, *vid, neigh));
        });
        let process_time = instant.elapsed().as_secs_f64();
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
    map: &CMap3<T>,
    graph: &[(VertexIdType, Vec<VertexIdType>)],
    n_rounds: usize,
    lambda: T,
) {
    println!(" Round | process_time | throughput(vertex/s)");
    let mut round = 0;
    let n_v = graph.len();
    loop {
        let instant = std::time::Instant::now();
        graph.par_iter().for_each(|(vid, neigh)| {
            atomically(|t| neighbor_based_smooth_3d(t, map, *vid, neigh, lambda));
        });
        let process_time = instant.elapsed().as_secs_f64();
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
    map: &CMap3<T>,
    graph: &[(VertexIdType, Vec<VertexIdType>)],
    n_rounds: usize,
    lambda: T,
    k: T,
) {
    println!(" Round | process_time | throughput(vertex/s)");
    let mut round = 0;
    let n_v = graph.len();
    let mu = T::one() / (k - T::one() / lambda);

    loop {
        let instant = std::time::Instant::now();
        let scale = if round % 2 == 0 { lambda } else { mu };
        graph.par_iter().for_each(|(vid, neigh)| {
            atomically(|t| neighbor_based_smooth_3d(t, map, *vid, neigh, scale));
        });
        let process_time = instant.elapsed().as_secs_f64();
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
