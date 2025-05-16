#[cfg(feature = "thread-binding")]
use std::sync::Arc;
use std::time::Instant;

use honeycomb::{
    core::{
        cmap::SewError,
        stm::{Transaction, TransactionControl, TransactionResult},
    },
    kernels::remeshing::{cut_inner_edge, cut_outer_edge},
    prelude::{CMap2, CMapBuilder, CoordsFloat, DartIdType, EdgeIdType},
};
#[cfg(feature = "thread-binding")]
use hwlocality::{Topology, cpu::binding::CpuBindingFlags};
use rayon::prelude::*;

#[cfg(feature = "thread-binding")]
use crate::utils::get_physical_cores;
use crate::{cli::CutEdgesArgs, utils::hash_file};

// const MAX_RETRY: u8 = 10;

pub fn bench_cut_edges<T: CoordsFloat>(
    args: CutEdgesArgs,
    n_threads: usize,
    bind_threads: bool,
) -> CMap2<T> {
    let input_map = args.input.to_str().unwrap();
    let target_len = T::from(args.target_length).unwrap();

    // load map from file
    let mut instant = Instant::now();
    let input_hash = hash_file(input_map).expect("E: could not compute input hash"); // file id for posterity

    let mut map: CMap2<T> = if input_map.ends_with(".cmap") {
        CMapBuilder::<2, _>::from_cmap_file(input_map)
            .build()
            .unwrap()
    } else if input_map.ends_with(".vtk") {
        CMapBuilder::<2, _>::from_vtk_file(input_map)
            .build()
            .unwrap()
    } else {
        panic!(
            "E: Unknown file format; only .cmap or .vtk files are supported for map initialization"
        );
    };
    #[cfg(debug_assertions)] // check input
    {
        use honeycomb::prelude::OrbitPolicy;
        assert!(
            map.iter_faces()
                .all(|f| { map.orbit(OrbitPolicy::Face, f as DartIdType).count() == 3 }),
            "Input mesh isn't a triangle mesh"
        );
    }
    println!("| cut-edges benchmark");
    println!("|-> input      : {input_map} (hash: {input_hash:#0x})",);
    println!(
        "|-> backend    : {:?} with {n_threads} thread(s)",
        args.backend
    );
    println!("|-> target size: {target_len:?}");
    println!("|-> init time  : {}ms", instant.elapsed().as_millis());

    println!(
        " Step | n_edge_total | n_edge_to_process | t_compute_batch(s) | t_process_batch(s) | n_transac_retry"
    );

    let mut step = 0;
    print!(" {step:>4} "); // Step

    // compute first batch
    instant = Instant::now();
    let mut edges: Vec<EdgeIdType> = map.iter_edges().collect();
    print!("| {:>12} ", edges.len()); // n_edge_total
    edges.retain(|&e| {
        let (vid1, vid2) = (
            map.vertex_id(e as DartIdType),
            map.vertex_id(map.beta::<1>(e as DartIdType)),
        );
        match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
            (Some(v1), Some(v2)) => (v2 - v1).norm() > target_len,
            (_, _) => false,
        }
    });
    let n_e = edges.len();
    print!("| {n_e:>17} "); // n_edge_to_process
    let mut nd = map.add_free_darts(6 * n_e); // 2 for edge split + 2*2 for new edges in neighbor tets
    let mut darts: Vec<DartIdType> = (nd..nd + 6 * n_e as DartIdType).collect();
    print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch

    // while there are edges to cut
    while !edges.is_empty() {
        // process batch
        instant = Instant::now();
        let n_retry = match args.backend {
            crate::cli::Backend::RayonIter => dispatch_rayon(&map, &mut edges, &darts),
            crate::cli::Backend::RayonChunks => {
                dispatch_rayon_chunks(&map, &mut edges, &darts, n_threads)
            }
            crate::cli::Backend::StdThreads => {
                dispatch_std_threads(&map, &mut edges, &darts, n_threads, bind_threads)
            }
        };
        print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_process_batch
        println!("| {n_retry:>15}",); // n_transac_retry

        (1..map.n_darts() as DartIdType).for_each(|d| {
            if map.is_free(d) && !map.is_unused(d) {
                map.remove_free_dart(d);
            }
        });

        // compute the new batch
        step += 1;
        print!(" {step:>4} "); // Step
        instant = Instant::now();
        edges.extend(map.iter_edges());
        print!("| {:>12} ", edges.len()); // n_edge_total
        edges.retain(|&e| {
            let (vid1, vid2) = (
                map.vertex_id(e as DartIdType),
                map.vertex_id(map.beta::<1>(e as DartIdType)),
            );
            match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
                (Some(v1), Some(v2)) => (v2 - v1).norm() > target_len,
                (_, _) => false,
            }
        });
        let n_e = edges.len();
        print!("| {n_e:>17} "); // n_edge_to_process
        nd = map.add_free_darts(6 * n_e);
        darts.par_drain(..); // is there a better way?
        darts.extend(nd..nd + 6 * n_e as DartIdType);
        if n_e != 0 {
            print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch
        } else {
            print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch
            print!("| {:>18.6e} ", 0.0); // t_process_batch
            println!("| {:>15}", 0); // n_transac_retry
        }
    }

    map
}

#[inline]
fn dispatch_rayon<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
) -> u32 {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    units
        .into_par_iter()
        .map(|(e, new_darts)| {
            let mut n_retry = 0;
            if map.is_i_free::<2>(e as DartIdType) {
                while !process_outer_edge(map, &mut n_retry, e, new_darts).is_validated() {}
            } else {
                while !process_inner_edge(map, &mut n_retry, e, new_darts).is_validated() {}
            }
            n_retry as u32
        }) // par_map
        .sum()
}

#[inline]
fn dispatch_rayon_chunks<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
    n_threads: usize,
) -> u32 {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    units
        .par_chunks(1 + units.len() / n_threads)
        .map(|c| {
            let mut n = 0;
            c.iter().for_each(|&(e, new_darts)| {
                let mut n_retry = 0;
                if map.is_i_free::<2>(e as DartIdType) {
                    while !process_outer_edge(map, &mut n_retry, e, new_darts).is_validated() {}
                } else {
                    while !process_inner_edge(map, &mut n_retry, e, new_darts).is_validated() {}
                }
                n += n_retry as u32;
            });
            n
        }) // par_for_each
        .sum()
}

#[inline]
fn dispatch_std_threads<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
    n_threads: usize,
    bind_threads: bool,
) -> u32 {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    std::thread::scope(|s| {
        if bind_threads {
            #[cfg(not(feature = "thread-binding"))]
            {
                unreachable!(); // due to CLI option only being present with the feature
            }
            #[cfg(feature = "thread-binding")]
            {
                let topology = Topology::new().unwrap();
                let topology = Arc::new(topology);
                match get_physical_cores(&topology) {
                    Ok(mut cores) => {
                        let n_t = if cores.len() < n_threads {
                            // don't allow more than one thread per physical core
                            // this is sane since this branch executes only if we explicitly enable binding
                            eprintln!(
                                "W: Less physical cores than logical threads; proceeding with one thread per core ({})",
                                cores.len()
                            );
                            cores.len()
                        } else {
                            n_threads
                        };
                        let mut handles = Vec::new();
                        for wl in units.chunks(1 + units.len() / n_t) {
                            let topology = topology.clone();
                            let core = cores.pop_front().expect("E: unreachable");
                            let mut bind_to = core.cpuset().unwrap().clone_target();
                            bind_to.singlify();
                            handles.push(s.spawn(move || {
                                // bind thread
                                let tid = hwlocality::current_thread_id();
                                topology
                                    .bind_thread_cpu(tid, &bind_to, CpuBindingFlags::empty())
                                    .unwrap();
                                // execute work
                                let mut n = 0;
                                wl.iter().for_each(|&(e, new_darts)| {
                                    let mut n_retry = 0;
                                    if map.is_i_free::<2>(e as DartIdType) {
                                        while !process_outer_edge(map, &mut n_retry, e, new_darts)
                                            .is_validated()
                                        {
                                        }
                                    } else {
                                        while !process_inner_edge(map, &mut n_retry, e, new_darts)
                                            .is_validated()
                                        {
                                        }
                                    }
                                    n += n_retry as u32;
                                });
                                n
                            })); // s.spawn
                        } // for wl in workloads
                        handles.into_iter().map(|h| h.join().unwrap()).sum()
                    }
                    Err(e) => {
                        eprintln!("W: {e}");
                        let mut handles = Vec::new();
                        for wl in units.chunks(1 + units.len() / n_threads) {
                            handles.push(s.spawn(|| {
                                let mut n = 0;
                                wl.iter().for_each(|&(e, new_darts)| {
                                    let mut n_retry = 0;
                                    if map.is_i_free::<2>(e as DartIdType) {
                                        while !process_outer_edge(map, &mut n_retry, e, new_darts)
                                            .is_validated()
                                        {
                                        }
                                    } else {
                                        while !process_inner_edge(map, &mut n_retry, e, new_darts)
                                            .is_validated()
                                        {
                                        }
                                    }
                                    n += n_retry as u32;
                                });
                                n
                            })); // s.spawn
                        } // for wl in workloads
                        handles.into_iter().map(|h| h.join().unwrap()).sum()
                    }
                }
            }
        } else {
            let mut handles = Vec::new();
            for wl in units.chunks(1 + units.len() / n_threads) {
                handles.push(s.spawn(|| {
                    let mut n = 0;
                    wl.iter().for_each(|&(e, new_darts)| {
                        let mut n_retry = 0;
                        if map.is_i_free::<2>(e as DartIdType) {
                            while !process_outer_edge(map, &mut n_retry, e, new_darts)
                                .is_validated()
                            {}
                        } else {
                            while !process_inner_edge(map, &mut n_retry, e, new_darts)
                                .is_validated()
                            {}
                        }
                        n += n_retry as u32;
                    });
                    n
                })); // s.spawn
            } // for wl in workloads
            handles.into_iter().map(|h| h.join().unwrap()).sum()
        }
    }) // std::thread::scope
}

#[inline]
fn process_outer_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    n_retry: &mut u8,
    e: EdgeIdType,
    [nd1, nd2, nd3, _, _, _]: [DartIdType; 6],
) -> TransactionResult<(), SewError> {
    Transaction::with_control_and_err(
        |_| {
            *n_retry += 1;
            TransactionControl::Retry
        },
        |trans| cut_outer_edge(trans, map, e, [nd1, nd2, nd3]),
    ) // Transaction::with_control
}

#[inline]
fn process_inner_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    n_retry: &mut u8,
    e: EdgeIdType,
    nds: [DartIdType; 6],
) -> TransactionResult<(), SewError> {
    Transaction::with_control_and_err(
        |_| {
            *n_retry += 1;
            TransactionControl::Retry
        },
        |trans| cut_inner_edge(trans, map, e, nds),
    ) // Transaction::with_control
}
