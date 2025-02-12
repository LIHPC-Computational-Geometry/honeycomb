use std::time::Instant;

use rayon::prelude::*;

use honeycomb::{
    core::cmap::SewError,
    core::stm::{
        retry, try_or_coerce, StmError, Transaction, TransactionControl, TransactionResult,
    },
    prelude::{
        CMap2, CMapBuilder, CoordsFloat, DartIdType, EdgeIdType, Orbit2, OrbitPolicy, Vertex2,
    },
};

const TARGET_LENGTH: f64 = 0.1;
const MAX_RETRY: u8 = 10;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_map = args.get(1).expect("E: no input file specified");
    // load map from file
    let mut instant = Instant::now();
    let mut map: CMap2<f64> = CMapBuilder::from(input_map).build().unwrap();
    println!("map built in {}ms", instant.elapsed().as_millis());

    #[cfg(debug_assertions)] // check input
    {
        instant = Instant::now();
        assert!(
            map.iter_faces()
                .all(|f| { Orbit2::new(&map, OrbitPolicy::Face, f as DartIdType).count() == 3 }),
            "Input mesh isn't a triangle mesh"
        );
        println!("topology checked in {}ms", instant.elapsed().as_millis());
    }

    let backend = if std::env::var("BACKEND").is_ok_and(|s| &s == "rayon") {
        Backend::Rayon
    } else if std::env::var("BACKEND").is_ok_and(|s| &s == "chunks") {
        Backend::RayonChunks
    } else {
        Backend::StdThreads
    };
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let mut step = 0;

    // compute first batch
    instant = Instant::now();
    let mut edges: Vec<EdgeIdType> = fetch_edges_to_process(&map, &TARGET_LENGTH).collect();
    let n_e = edges.len();
    let mut nd = map.add_free_darts(6 * n_e); // 2 for edge split + 2*2 for new edges in neighbor tets
    let mut darts: Vec<DartIdType> = (nd..nd + 6 * n_e as DartIdType).collect();
    println!(
        "[B{}] computed in {}ms | {n_e} edges to process",
        step,
        instant.elapsed().as_millis()
    );

    // while there are edges to cut
    while !edges.is_empty() {
        // process batch
        instant = Instant::now();
        match backend {
            Backend::Rayon => dispatch_rayon(&map, &mut edges, &darts),
            Backend::RayonChunks => dispatch_rayon_chunks(&map, &mut edges, &darts, n_threads),
            Backend::StdThreads => dispatch_std_threads(&map, &mut edges, &darts, n_threads),
        };
        println!(
            "[B{}] processed in {}ms",
            step,
            instant.elapsed().as_millis()
        );

        (1..map.n_darts() as DartIdType).for_each(|d| {
            if map.is_free(d) && !map.is_unused(d) {
                map.remove_free_dart(d);
            }
        });

        // compute the new batch
        instant = Instant::now();
        step += 1;
        // TRADEOFF: par compute & reallocate each round vs eq compute & extend
        // edges = fetch_edges_to_process(&map, &TARGET_LENGTH).collect();
        edges.extend(fetch_edges_to_process(&map, &TARGET_LENGTH));
        let n_e = edges.len();
        nd = map.add_free_darts(6 * n_e);
        darts.par_drain(..); // is there a better way?
        darts.extend(nd..nd + 6 * n_e as DartIdType);
        if n_e != 0 {
            println!(
                "[B{}] computed in {}ms | {n_e} edges to process",
                step,
                instant.elapsed().as_millis()
            );
        }
    }

    #[cfg(debug_assertions)] // check output
    {
        assert!(map
            .iter_edges()
            .filter_map(|e| {
                let (vid1, vid2) = (
                    map.vertex_id(e as DartIdType),
                    map.vertex_id(map.beta::<1>(e as DartIdType)),
                );
                match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
                    (Some(v1), Some(v2)) => Some((v2 - v1).norm()),
                    (_, _) => None,
                }
            })
            .all(|norm| norm <= TARGET_LENGTH));
        assert!(map
            .iter_vertices()
            .all(|v| map.force_read_vertex(v).is_some()));
        assert!(
            map.iter_faces()
                .all(|f| { Orbit2::new(&map, OrbitPolicy::Face, f as DartIdType).count() == 3 }),
            "Input mesh isn't a triangle mesh"
        );
    }

    std::hint::black_box(map);
}

fn fetch_edges_to_process<'a, 'b, T: CoordsFloat>(
    map: &'a CMap2<T>,
    length: &'b T,
) -> impl Iterator<Item = EdgeIdType> + 'a
where
    'b: 'a,
{
    map.iter_edges().filter(|&e| {
        let (vid1, vid2) = (
            map.vertex_id(e as DartIdType),
            map.vertex_id(map.beta::<1>(e as DartIdType)),
        );
        match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
            (Some(v1), Some(v2)) => (v2 - v1).norm() > *length,
            (_, _) => false,
        }
    })
}

enum Backend {
    Rayon,
    RayonChunks,
    StdThreads,
}

#[inline]
fn dispatch_rayon<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
) {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    units.into_par_iter().for_each(|(e, new_darts)| {
        let mut n_retry = 0;
        if map.is_i_free::<2>(e as DartIdType) {
            let _ = process_outer_edge(map, &mut n_retry, e, new_darts);
        } else {
            let _ = process_inner_edge(map, &mut n_retry, e, new_darts);
        }
    }); // par_for_each
}

#[inline]
fn dispatch_rayon_chunks<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
    n_threads: usize,
) {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    units.par_chunks(1 + units.len() / n_threads).for_each(|c| {
        let wl = c.to_vec(); // allocating here effectively moves the data to the thread
        wl.into_iter().for_each(|(e, new_darts)| {
            let mut n_retry = 0;
            if map.is_i_free::<2>(e as DartIdType) {
                let _ = process_outer_edge(map, &mut n_retry, e, new_darts);
            } else {
                let _ = process_inner_edge(map, &mut n_retry, e, new_darts);
            }
        })
    }); // par_for_each
}

#[inline]
fn dispatch_std_threads<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
    n_threads: usize,
) {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    std::thread::scope(|s| {
        for wl in units.chunks(1 + units.len() / n_threads) {
            s.spawn(|| {
                let wl = wl.to_vec(); // allocating here effectively moves the data to the thread
                wl.into_iter().for_each(|(e, new_darts)| {
                    let mut n_retry = 0;
                    if map.is_i_free::<2>(e as DartIdType) {
                        let _ = process_outer_edge(map, &mut n_retry, e, new_darts);
                    } else {
                        let _ = process_inner_edge(map, &mut n_retry, e, new_darts);
                    }
                });
            }); // s.spawn
        } // for wl in workloads
    }); // std::thread::scope
}

#[inline]
fn process_outer_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    n_retry: &mut u8,
    e: EdgeIdType,
    [nd1, nd2, nd3, _, _, _]: [DartIdType; 6],
) -> TransactionResult<(), SewError> {
    Transaction::with_control_and_err(
        |e| match e {
            StmError::Failure => TransactionControl::Abort,
            StmError::Retry => {
                if *n_retry < MAX_RETRY {
                    *n_retry += 1;
                    TransactionControl::Retry
                } else {
                    TransactionControl::Abort
                }
            }
        },
        |trans| {
            try_or_coerce!(map.link::<2>(trans, nd1, nd2), SewError);
            try_or_coerce!(map.link::<1>(trans, nd2, nd3), SewError);

            let ld = e as DartIdType;
            let (b0ld, b1ld) = (
                map.beta_transac::<0>(trans, ld)?,
                map.beta_transac::<1>(trans, ld)?,
            );

            let (vid1, vid2) = (
                map.vertex_id_transac(trans, ld)?,
                map.vertex_id_transac(trans, b1ld)?,
            );
            let new_v = Vertex2::average(
                &map.read_vertex(trans, vid1)
                    .map_err(|_| StmError::Retry)?
                    .unwrap(),
                &map.read_vertex(trans, vid2)
                    .map_err(|_| StmError::Retry)?
                    .unwrap(),
            );
            map.write_vertex(trans, nd1, new_v)?;

            map.unsew::<1>(trans, ld).map_err(|_| StmError::Retry)?;
            map.unsew::<1>(trans, b1ld).map_err(|_| StmError::Retry)?;

            map.sew::<1>(trans, ld, nd1).map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, nd1, b0ld)
                .map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, nd3, b1ld)
                .map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, b1ld, nd2)
                .map_err(|_| StmError::Retry)?;

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
) -> TransactionResult<(), SewError> {
    Transaction::with_control_and_err(
        |e| match e {
            StmError::Failure => TransactionControl::Abort,
            StmError::Retry => {
                if *n_retry < MAX_RETRY {
                    *n_retry += 1;
                    TransactionControl::Retry
                } else {
                    TransactionControl::Abort
                }
            }
        },
        |trans| {
            try_or_coerce!(map.link::<2>(trans, nd1, nd2), SewError);
            try_or_coerce!(map.link::<1>(trans, nd2, nd3), SewError);
            try_or_coerce!(map.link::<2>(trans, nd4, nd5), SewError);
            try_or_coerce!(map.link::<1>(trans, nd5, nd6), SewError);

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
            let new_v = match (
                map.read_vertex(trans, vid1).map_err(|_| StmError::Retry)?,
                map.read_vertex(trans, vid2).map_err(|_| StmError::Retry)?,
            ) {
                (Some(v1), Some(v2)) => Vertex2::average(&v1, &v2),
                _ => retry()?,
            };
            map.write_vertex(trans, nd1, new_v)
                .map_err(|_| StmError::Retry)?;

            map.unsew::<2>(trans, ld).map_err(|_| StmError::Retry)?;
            map.unsew::<1>(trans, ld).map_err(|_| StmError::Retry)?;
            map.unsew::<1>(trans, b1ld).map_err(|_| StmError::Retry)?;
            map.unsew::<1>(trans, rd).map_err(|_| StmError::Retry)?;
            map.unsew::<1>(trans, b1rd).map_err(|_| StmError::Retry)?;

            map.sew::<2>(trans, ld, nd6).map_err(|_| StmError::Retry)?;
            map.sew::<2>(trans, rd, nd3).map_err(|_| StmError::Retry)?;

            map.sew::<1>(trans, ld, nd1).map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, nd1, b0ld)
                .map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, nd3, b1ld)
                .map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, b1ld, nd2)
                .map_err(|_| StmError::Retry)?;

            map.sew::<1>(trans, rd, nd4).map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, nd4, b0rd)
                .map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, nd6, b1rd)
                .map_err(|_| StmError::Retry)?;
            map.sew::<1>(trans, b1rd, nd5)
                .map_err(|_| StmError::Retry)?;

            Ok(())
        },
    ) // Transaction::with_control
}
