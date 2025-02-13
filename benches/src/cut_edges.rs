use std::time::Instant;

use rayon::prelude::*;

use honeycomb::{
    core::cmap::LinkError,
    core::stm::{retry, StmError, Transaction, TransactionControl, TransactionResult},
    prelude::{CMap2, CMapBuilder, CoordsFloat, DartIdType, EdgeIdType, Vertex2},
};

use crate::{cli::CutEdgesArgs, utils::hash_file};

// const MAX_RETRY: u8 = 10;

pub fn bench_cut_edges<T: CoordsFloat>(args: CutEdgesArgs) -> CMap2<T> {
    let input_map = args.input.to_str().unwrap();
    let target_len = T::from(args.target_length).unwrap();

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    // load map from file
    let mut instant = Instant::now();
    let input_hash = hash_file(input_map).expect("E: could not compute input hash"); // file id for posterity
    let mut map: CMap2<T> = CMapBuilder::from(input_map).build().unwrap();
    #[cfg(debug_assertions)] // check input
    {
        use honeycomb::prelude::{Orbit2, OrbitPolicy};
        assert!(
            map.iter_faces()
                .all(|f| { Orbit2::new(&map, OrbitPolicy::Face, f as DartIdType).count() == 3 }),
            "Input mesh isn't a triangle mesh"
        );
    }
    println!("Run information");
    println!("|-> input      : {input_map} (hash: {input_hash:#0x})",);
    println!(
        "|-> backend    : {:?} with {n_threads} thread(s)",
        args.backend
    );
    println!("|-> target size: {target_len:?}");
    println!("|-> init time  : {}ms", instant.elapsed().as_millis());

    println!(" Step | n_edge_total | n_edge_to_process | t_compute_batch(s) | t_process_batch(s) | n_transac_retry");

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
                dispatch_std_threads(&map, &mut edges, &darts, n_threads)
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
            println!("| {:>18.6e} ", 0.0); // t_process_batch
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
                if !process_outer_edge(map, &mut n_retry, e, new_darts).is_validated() {
                    unreachable!()
                }
            } else if !process_inner_edge(map, &mut n_retry, e, new_darts).is_validated() {
                unreachable!()
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
                    if !process_outer_edge(map, &mut n_retry, e, new_darts).is_validated() {
                        unreachable!()
                    }
                } else if !process_inner_edge(map, &mut n_retry, e, new_darts).is_validated() {
                    unreachable!()
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
    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for wl in units.chunks(1 + units.len() / n_threads) {
            handles.push(s.spawn(|| {
                let mut n = 0;
                wl.iter().for_each(|&(e, new_darts)| {
                    let mut n_retry = 0;
                    if map.is_i_free::<2>(e as DartIdType) {
                        if !process_outer_edge(map, &mut n_retry, e, new_darts).is_validated() {
                            unreachable!()
                        }
                    } else if !process_inner_edge(map, &mut n_retry, e, new_darts).is_validated() {
                        unreachable!()
                    }
                    n += n_retry as u32;
                });
                n
            })); // s.spawn
        } // for wl in workloads
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    }) // std::thread::scope
}

#[inline]
fn process_outer_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    n_retry: &mut u8,
    e: EdgeIdType,
    [nd1, nd2, nd3, _, _, _]: [DartIdType; 6],
) -> TransactionResult<(), LinkError> {
    Transaction::with_control_and_err(
        |_| {
            *n_retry += 1;
            TransactionControl::Retry
        },
        |trans| {
            // unfallible
            map.link::<2>(trans, nd1, nd2)?;
            map.link::<1>(trans, nd2, nd3)?;

            let ld = e as DartIdType;
            let (b0ld, b1ld) = (
                map.beta_transac::<0>(trans, ld)?,
                map.beta_transac::<1>(trans, ld)?,
            );

            let (vid1, vid2) = (
                map.vertex_id_transac(trans, ld)?,
                map.vertex_id_transac(trans, b1ld)?,
            );
            let new_v = match (map.read_vertex(trans, vid1)?, map.read_vertex(trans, vid2)?) {
                (Some(v1), Some(v2)) => Vertex2::average(&v1, &v2),
                _ => retry()?,
            };
            map.write_vertex(trans, nd1, new_v)?;

            map.unsew::<1>(trans, ld).map_err(|_| StmError::Failure)?;
            map.unsew::<1>(trans, b1ld).map_err(|_| StmError::Failure)?;

            map.sew::<1>(trans, ld, nd1)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, nd1, b0ld)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, nd3, b1ld)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, b1ld, nd2)
                .map_err(|_| StmError::Failure)?;

            Ok(())
        },
    ) // Transaction::with_control
}

#[inline]
fn process_inner_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    n_retry: &mut u8,
    e: EdgeIdType,
    [nd1, nd2, nd3, nd4, nd5, nd6]: [DartIdType; 6],
) -> TransactionResult<(), LinkError> {
    Transaction::with_control_and_err(
        |_| {
            *n_retry += 1;
            TransactionControl::Retry
        },
        |trans| {
            // unfallible
            map.link::<2>(trans, nd1, nd2)?;
            map.link::<1>(trans, nd2, nd3)?;
            map.link::<2>(trans, nd4, nd5)?;
            map.link::<1>(trans, nd5, nd6)?;

            let (ld, rd) = (
                e as DartIdType,
                map.beta_transac::<2>(trans, e as DartIdType)?,
            );
            let (b0ld, b1ld) = (
                map.beta_transac::<0>(trans, ld)?,
                map.beta_transac::<1>(trans, ld)?,
            );
            let (b0rd, b1rd) = (
                map.beta_transac::<0>(trans, rd)?,
                map.beta_transac::<1>(trans, rd)?,
            );

            let (vid1, vid2) = (
                map.vertex_id_transac(trans, ld)?,
                map.vertex_id_transac(trans, b1ld)?,
            );
            let new_v = match (map.read_vertex(trans, vid1)?, map.read_vertex(trans, vid2)?) {
                (Some(v1), Some(v2)) => Vertex2::average(&v1, &v2),
                _ => retry()?,
            };
            map.write_vertex(trans, nd1, new_v)?;

            map.unsew::<2>(trans, ld).map_err(|_| StmError::Failure)?;
            map.unsew::<1>(trans, ld).map_err(|_| StmError::Failure)?;
            map.unsew::<1>(trans, b1ld).map_err(|_| StmError::Failure)?;
            map.unsew::<1>(trans, rd).map_err(|_| StmError::Failure)?;
            map.unsew::<1>(trans, b1rd).map_err(|_| StmError::Failure)?;

            map.sew::<2>(trans, ld, nd6)
                .map_err(|_| StmError::Failure)?;
            map.sew::<2>(trans, rd, nd3)
                .map_err(|_| StmError::Failure)?;

            map.sew::<1>(trans, ld, nd1)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, nd1, b0ld)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, nd3, b1ld)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, b1ld, nd2)
                .map_err(|_| StmError::Failure)?;

            map.sew::<1>(trans, rd, nd4)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, nd4, b0rd)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, nd6, b1rd)
                .map_err(|_| StmError::Failure)?;
            map.sew::<1>(trans, b1rd, nd5)
                .map_err(|_| StmError::Failure)?;

            Ok(())
        },
    ) // Transaction::with_control
}
