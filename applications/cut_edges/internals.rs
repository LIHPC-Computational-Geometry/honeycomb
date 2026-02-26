use std::time::Instant;

use honeycomb::{
    prelude::{
        CMap2, CoordsFloat, DartIdType, EdgeIdType, SewError,
        remeshing::{cut_inner_edge, cut_outer_edge},
    },
    stm::{Transaction, TransactionControl, TransactionResult},
};
use rayon::prelude::*;

use applications::{Backend, prof_start, prof_stop};

// -- main loop

pub fn cut_edges<T: CoordsFloat>(
    map: &mut CMap2<T>,
    target_length: T,
    backend: Backend,
    n_threads: usize,
) {
    println!(
        " Step | n_edge_total | n_edge_to_process | t_compute_batch(s) | t_process_batch(s) | n_tx_retry"
    );

    let mut step = 0;
    print!(" {step:>4} "); // Step
    prof_start!("HCBENCH_CUTS");

    // compute first batch
    let n_e_total = map.par_iter_edges().count();
    prof_start!("HCBENCH_CUTS_COMPUTE");
    let mut instant = Instant::now();
    let mut edges: Vec<EdgeIdType> = map
        .par_iter_edges()
        .filter(|&e| {
            let (vid1, vid2) = (
                map.vertex_id(e as DartIdType),
                map.vertex_id(map.beta::<1>(e as DartIdType)),
            );
            match (map.read_vertex(vid1), map.read_vertex(vid2)) {
                (Some(v1), Some(v2)) => (v2 - v1).norm() > target_length,
                (_, _) => false,
            }
        })
        .collect();
    let n_e = edges.len();
    print!("| {n_e_total:>12} ");
    print!("| {n_e:>17} ");
    let mut nd = map.allocate_used_darts(6 * n_e); // 6 darts needed per cut
    let mut darts: Vec<DartIdType> = (nd..nd + 6 * n_e as DartIdType).into_par_iter().collect();
    prof_stop!("HCBENCH_CUTS_COMPUTE");
    print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch

    // while there are edges to cut
    while !edges.is_empty() {
        // process batch
        prof_start!("HCBENCH_CUTS_PROCESS");
        instant = Instant::now();
        let n_retry = match backend {
            Backend::RayonIter => dispatch_rayon(map, &mut edges, &darts),
            Backend::RayonChunks => dispatch_rayon_chunks(map, &mut edges, &darts, n_threads),
            Backend::StdThreads => dispatch_std_threads(map, &mut edges, &darts, n_threads),
        };
        prof_stop!("HCBENCH_CUTS_PROCESS");
        print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_process_batch
        println!("| {n_retry:>15}",); // n_tx_retry

        (1..map.n_darts() as DartIdType).for_each(|d| {
            if map.is_free(d) && !map.is_unused(d) {
                map.release_dart(d).expect("E: unreachable");
            }
        });

        // compute the new batch
        step += 1;
        let n_e_total = map.par_iter_edges().count();
        print!(" {step:>4} "); // Step
        prof_start!("HCBENCH_CUTS_COMPUTE");
        instant = Instant::now();
        edges.par_extend(map.par_iter_edges().filter(|&e| {
            let (vid1, vid2) = (
                map.vertex_id(e as DartIdType),
                map.vertex_id(map.beta::<1>(e as DartIdType)),
            );
            match (map.read_vertex(vid1), map.read_vertex(vid2)) {
                (Some(v1), Some(v2)) => (v2 - v1).norm() > target_length,
                (_, _) => false,
            }
        }));
        print!("| {n_e_total:>12} ");
        let n_e = edges.len();
        print!("| {n_e:>17} ");
        nd = map.allocate_used_darts(6 * n_e);
        darts.par_drain(..); // is there a better way?
        darts.extend(nd..nd + 6 * n_e as DartIdType);
        prof_stop!("HCBENCH_CUTS_COMPUTE");
        if n_e != 0 {
            print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch
        } else {
            print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch
            print!("| {:>18.6e} ", 0.0); // t_process_batch
            println!("| {:>15}", 0); // n_tx_retry
        }
    }
    prof_stop!("HCBENCH_CUTS");
}

// -- dispatch

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
) -> u32 {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();

    #[cfg(feature = "bind-threads")]
    {
        use std::sync::Arc;

        use hwlocality::{Topology, cpu::binding::CpuBindingFlags};

        use applications::get_proc_list;

        let topo = Arc::new(Topology::new().unwrap());
        let mut cores = get_proc_list(&topo).unwrap_or_default().into_iter().cycle();
        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for wl in units.chunks(1 + units.len() / n_threads) {
                let topo = topo.clone();
                let core = cores.next();
                handles.push(s.spawn(move || {
                    // bind
                    if let Some(c) = core {
                        let tid = hwlocality::current_thread_id();
                        topo.bind_thread_cpu(tid, &c, CpuBindingFlags::empty())
                            .unwrap();
                    }
                    // work
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
        }) // std::thread::scope
    }

    #[cfg(not(feature = "bind-threads"))]
    {
        std::thread::scope(|s| {
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
        }) // std::thread::scope
    }
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
        |t| cut_outer_edge(t, map, e, [nd1, nd2, nd3]),
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
        |t| cut_inner_edge(t, map, e, nds),
    ) // Transaction::with_control
}
