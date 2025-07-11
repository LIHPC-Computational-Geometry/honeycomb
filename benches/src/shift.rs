//! `shift` benchmark
//!
//! The benchmark fetches all vertices that are not on the border of the map, fetch all identifiers
//! of each respective vertices' neighbors. Then, for all vertices:
//!
//! - compute the average between neighbors,
//! - overwrite current vertex value with computed average.
//!
//! This is done for a given number of rounds, to reach significant execution time and an accurate
//! average.

use honeycomb::kernels::remeshing::move_vertex_to_average;
use rayon::prelude::*;

use honeycomb::core::stm::{Transaction, TransactionControl};
use honeycomb::prelude::{
    CMap2, CMapBuilder, CoordsFloat, DartIdType, NULL_DART_ID, OrbitPolicy, VertexIdType,
};

use crate::cli::ShiftArgs;
use crate::utils::hash_file;
use crate::{prof_start, prof_stop};

pub fn bench_shift<T: CoordsFloat>(args: ShiftArgs) -> CMap2<T> {
    let mut instant = std::time::Instant::now();
    let input_map = args.input.to_str().unwrap();
    let input_hash = hash_file(input_map).unwrap();
    let map: CMap2<T> = if input_map.ends_with(".cmap") {
        CMapBuilder::<2, T>::from_cmap_file(input_map)
            .build()
            .unwrap()
    } else if input_map.ends_with(".vtk") {
        CMapBuilder::<2, T>::from_vtk_file(input_map)
            .build()
            .unwrap()
    } else {
        panic!(
            "E: Unknown file format; only .cmap or .vtk files are supported for map initialization"
        );
    };
    let build_time = instant.elapsed();

    prof_start!("HCBENCH_SHIFT");
    if args.no_conflict {
        todo!("TODO: require a partitioning algorithm")
    } else {
        instant = std::time::Instant::now();
        // fetch all vertices that are not on the boundary of the map
        let tmp: Vec<(VertexIdType, Vec<VertexIdType>)> = map
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
        let n_v = tmp.len();
        let graph_time = instant.elapsed();
        let n_threads = rayon::current_num_threads();

        println!("| shift benchmark");
        println!("|-> input      : {input_map} (hash: {input_hash:#0x})");
        println!("|-> backend    : rayon-iter with {n_threads} thread(s)",);
        println!("|-> # of rounds: {}", args.n_rounds.get());
        println!("|-+ init time  :");
        println!("| |->   map built in {}ms", build_time.as_millis());
        println!("| |-> graph built in {}ms", graph_time.as_millis());

        println!(" Round | process_time | throughput(vertex/s) | n_transac_retry");
        // main loop
        let mut round = 0;
        let mut process_time;
        loop {
            instant = std::time::Instant::now();
            let n_retry: u32 = tmp
                .par_iter()
                .map(|(vid, neigh)| {
                    let mut n = 0;
                    Transaction::with_control(
                        |_| {
                            n += 1;
                            TransactionControl::Retry
                        },
                        |trans| move_vertex_to_average(trans, &map, *vid, neigh),
                    );
                    n
                })
                .sum();
            process_time = instant.elapsed().as_secs_f64();
            println!(
                " {:>5} | {:>12.6e} | {:>20.6e} | {:>15}",
                round,
                process_time,
                n_v as f64 / process_time,
                n_retry,
            );

            round += 1;
            if round >= args.n_rounds.get() {
                break;
            }
        }
    }
    prof_stop!("HCBENCH_SHIFT");

    map
}
