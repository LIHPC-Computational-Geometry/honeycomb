mod cavity;
mod delaunay;
mod sample;

use std::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use coupe::nalgebra::Matrix3;
use honeycomb::{
    core::{
        cmap::{CMap3, DartIdType, VolumeIdType},
        geometry::{CoordsFloat, Vertex3},
        stm::{Transaction, abort, atomically_with_err, try_or_coerce},
    },
    prelude::{NULL_DART_ID, grid_generation::GridBuilder},
    stm::{TVar, TransactionClosureResult, unwrap_or_abort},
};
use rayon::prelude::*;
use rustc_hash::FxHashSet as HashSet;

use cavity::{
    CavityError, DART_BLOCK_START, carve_cavity_3d, extend_to_starshaped_cavity_3d,
    rebuild_cavity_3d,
};
use delaunay::{DelaunayError, compute_delaunay_cavity_3d};

use crate::internals::sample::{compute_brio, sample_points};

thread_local! {
    pub static LAST_INSERTED: Cell<VolumeIdType> = const { Cell::new(1) };
}

#[allow(clippy::too_many_arguments)]
pub fn delaunay_box_3d<T: CoordsFloat>(
    lx: f64,
    ly: f64,
    lz: f64,
    n_points: usize,
    seed: u64,
    probability: f64,
) -> CMap3<T> {
    assert!(lx > 0.0);
    assert!(ly > 0.0);
    assert!(lz > 0.0);
    assert!(n_points > 0);

    let n_threads = rayon::current_num_threads();
    println!("| Delaunay box triangulation benchmark");
    println!("|-> sampling domain: [0;{lx}]x[0;{ly}]x[0;{lz}]");
    println!("|-> point inserts  : {n_points}");
    println!("|-> threads used   : {n_threads}");

    let mut instant = Instant::now();
    let points: Vec<_> = sample_points(lx, ly, lz, n_points, seed).collect();
    println!(
        "|-> sampling time  : {:>8.3e}",
        instant.elapsed().as_secs_f32()
    );

    instant = Instant::now();
    let (brio_r1, brs) = compute_brio::<T>(points, seed, probability);
    println!(
        "|-> BRIO time      : {:>8.3e}",
        instant.elapsed().as_secs_f32()
    );

    let mut map = GridBuilder::<3, _>::default()
        .n_cells([1, 1, 1])
        .lens([
            T::from(lx).unwrap(),
            T::from(ly).unwrap(),
            T::from(lz).unwrap(),
        ])
        .split_cells(true)
        .build()
        .unwrap();

    instant = Instant::now();
    // typical point distribution will result in 6-8*n_points tets
    // 20 gives some leeway, 12 is the number of darts per tet
    let n_alloc = 20 * 12 * n_points;
    let start = map.allocate_unused_darts(n_alloc);
    // initialize the search offset for dart reservations
    let block_size = (20 * 12 * n_points) / rayon::current_num_threads();
    rayon::broadcast(|_| {
        let tid = rayon::current_thread_index().expect("E: unreachable");
        DART_BLOCK_START.set(TVar::new(start + (tid * block_size) as DartIdType));
    });
    println!(
        "|-> init time      : {:>8.3e}",
        instant.elapsed().as_secs_f32()
    );

    println!(" BRIO round | total inserts | successful inserts | time (s) | throughput (p/s)",);
    instant = Instant::now();
    let mut count = 0;
    let r1n = brio_r1.len();
    brio_r1.into_iter().for_each(|p| {
        loop {
            match atomically_with_err(|t| insert_points(t, &map, p)) {
                Ok(()) => {
                    count += 1;
                    break;
                }
                Err(e) => match e {
                    DelaunayError::CircumsphereSingularity => break,
                    DelaunayError::CavityBuilding(e) => match e {
                        CavityError::FailedOp(_)
                        | CavityError::FailedReservation(_)
                        | CavityError::InconsistentState(_) => {
                            continue;
                        }
                        CavityError::FailedRelease(_) | CavityError::NonExtendable(_) => {
                            break;
                        }
                    },
                },
            }
        }
    });
    let time = instant.elapsed().as_secs_f32();
    println!(
        " {:>10} | {:>13} | {:>18} | {:>8.3e} | {:>10.3e}",
        1,
        r1n,
        count,
        time,
        count as f32 / time,
    );

    if let Some(brio_rs) = brs {
        let mut round_idx = 1;
        for round in brio_rs {
            instant = Instant::now();
            round_idx += 1;
            let rnn = round.len();
            let counters: Vec<AtomicUsize> = (0..n_threads).map(|_| AtomicUsize::new(0)).collect();
            round.into_par_iter().for_each(|p| {
                // round.par_chunks(round.len().div_ceil(4)).for_each(|c| {
                //     c.into_iter().for_each(|&p| {
                loop {
                    match atomically_with_err(|t| insert_points(t, &map, p)) {
                        Ok(()) => {
                            let tid = rayon::current_thread_index().expect("E: unreachable");
                            counters[tid].fetch_add(1, Ordering::Relaxed);
                            break;
                        }
                        Err(e) => {
                            // eprintln!("E: insertion failed - {e}");
                            match e {
                                DelaunayError::CircumsphereSingularity => break,
                                DelaunayError::CavityBuilding(e) => match e {
                                    CavityError::FailedOp(_)
                                    | CavityError::FailedReservation(_)
                                    | CavityError::InconsistentState(_) => {
                                        continue;
                                    }
                                    CavityError::FailedRelease(_)
                                    | CavityError::NonExtendable(_) => {
                                        break;
                                    }
                                },
                            }
                        }
                    }
                }
                // });
            });
            let time = instant.elapsed().as_secs_f32();
            let count: usize = counters.iter().map(|c| c.load(Ordering::Relaxed)).sum();
            println!(
                " {:>10} | {:>13} | {:>18} | {:>8.3e} | {:>10.3e}",
                round_idx,
                rnn,
                count,
                time,
                count as f32 / time,
            );
        }
    }

    map
}

fn insert_points<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    p: Vertex3<T>,
) -> TransactionClosureResult<(), DelaunayError> {
    let start = if map.is_unused_tx(t, LAST_INSERTED.get())? {
        1 // technically this could be unused too
    } else {
        LAST_INSERTED.get()
    };
    let location = try_or_coerce!(locate_containing_tet(t, map, start, p), DelaunayError);
    let volume = match location {
        LocateResult::Found(v) => v,
        LocateResult::ReachedBoundary => {
            abort(DelaunayError::CavityBuilding(
                // NOTE:
                CavityError::InconsistentState("Points is beyond a boundary"),
            ))?;
            unreachable!();
        }
        LocateResult::Oscillating => {
            abort(DelaunayError::CavityBuilding(
                // NOTE:
                CavityError::InconsistentState("Non-ending search"),
            ))?;
            unreachable!();
        }
    };

    #[cfg(debug_assertions)]
    try_or_coerce!(check_tet_orientation(t, map, volume, p), DelaunayError);

    // compute cavity
    let cavity = compute_delaunay_cavity_3d(t, map, volume, p)?;
    // carve
    let carved_cavity = try_or_coerce!(carve_cavity_3d(t, map, cavity), DelaunayError);
    // extend
    let cavity = try_or_coerce!(
        extend_to_starshaped_cavity_3d(t, map, carved_cavity),
        DelaunayError
    );
    // rebuild
    let last_inserted = try_or_coerce!(rebuild_cavity_3d(t, map, cavity), DelaunayError);
    LAST_INSERTED.set(last_inserted);
    Ok(())
}

#[rustfmt::skip]
pub fn compute_tet_orientation<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    (d1, d2, d3): (DartIdType, DartIdType, DartIdType),
    p: Vertex3<T>,
) -> TransactionClosureResult<f64, CavityError> {
    let v1 = {
        let vid = map.vertex_id_tx(t, d1)?;
        unwrap_or_abort(map.read_vertex_tx(t, vid)?, CavityError::InconsistentState("Topological vertices have missing coordinates"))?
    };
    let v2 = {
        let vid = map.vertex_id_tx(t, d2)?;
        unwrap_or_abort(map.read_vertex_tx(t, vid)?, CavityError::InconsistentState("Topological vertices have missing coordinates"))?
    };
    let v3 = {
        let vid = map.vertex_id_tx(t, d3)?;
        unwrap_or_abort(map.read_vertex_tx(t, vid)?, CavityError::InconsistentState("Topological vertices have missing coordinates"))?
    };

    let c1 = (v1 - p).to_f64().expect("E: unreachable");
    let c2 = (v2 - p).to_f64().expect("E: unreachable");
    let c3 = (v3 - p).to_f64().expect("E: unreachable");

    Ok(Matrix3::from_column_slice(&[
        c1.x(), c1.y(), c1.z(), 
        c2.x(), c2.y(), c2.z(), 
        c3.x(), c3.y(), c3.z(), 
    ]).determinant())
}

enum LocateResult {
    Found(VolumeIdType),
    ReachedBoundary,
    Oscillating,
}

fn locate_containing_tet<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    start: VolumeIdType,
    p: Vertex3<T>,
) -> TransactionClosureResult<LocateResult, CavityError> {
    fn locate_next_tet<T: CoordsFloat>(
        t: &mut Transaction,
        map: &CMap3<T>,
        d: DartIdType,
        p: Vertex3<T>,
    ) -> TransactionClosureResult<Option<DartIdType>, CavityError> {
        let face_darts = [
            d as DartIdType,
            { map.beta_tx::<2>(t, d)? },
            {
                let b1 = map.beta_tx::<1>(t, d)?;
                map.beta_tx::<2>(t, b1)?
            },
            {
                let b0 = map.beta_tx::<0>(t, d)?;
                map.beta_tx::<2>(t, b0)?
            },
        ];

        let mut min = 0.0;
        let mut d_min = NULL_DART_ID;

        for d in face_darts {
            let b1 = map.beta_tx::<1>(t, d)?;
            let b0 = map.beta_tx::<0>(t, d)?;

            let orientation = compute_tet_orientation(t, map, (d, b1, b0), p)?;
            if orientation < min {
                min = orientation;
                d_min = map.beta_tx::<3>(t, d)?;
            }
        }
        if d_min == NULL_DART_ID {
            Ok(None)
        } else {
            Ok(Some(d_min))
        }
    }

    // TODO: find a better way to handle this
    let mut visited = HashSet::default();
    let mut count = 0;
    let max_count = map.n_darts() / 12;
    let mut count_visit = 0;
    let max_revisit = 2;
    let mut dart = start as DartIdType;
    visited.insert(map.face_id_tx(t, dart)?);

    loop {
        count += 1;
        if count_visit > max_revisit || count > max_count {
            return Ok(LocateResult::Oscillating);
        }
        if let Some(next_dart) = locate_next_tet(t, map, dart, p)? {
            dart = next_dart;
            if !visited.insert(map.face_id_tx(t, dart)?) {
                count_visit += 1;
            }
            // point is outside or across a gap in the mesh
            if dart == NULL_DART_ID {
                // it is possible to look for another path, but it requires a more complex condition
                // than "just follow the first neg volume direction"
                return Ok(LocateResult::ReachedBoundary);
            }
        } else {
            return Ok(LocateResult::Found(map.volume_id_tx(t, dart)?));
        }
    }
}

#[cfg(debug_assertions)]
fn check_tet_orientation<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    volume: VolumeIdType,
    p: Vertex3<T>,
) -> TransactionClosureResult<(), CavityError> {
    let f1 = (
        volume as DartIdType,
        map.beta_tx::<1>(t, volume as DartIdType)?,
        map.beta_tx::<0>(t, volume as DartIdType)?,
    );
    let f2 = {
        let d = map.beta_tx::<2>(t, volume as DartIdType)?;
        (d, map.beta_tx::<1>(t, d)?, map.beta_tx::<0>(t, d)?)
    };
    let f3 = {
        let d = map.beta_tx::<1>(t, volume as DartIdType)?;
        let b2 = map.beta_tx::<2>(t, d)?;
        (b2, map.beta_tx::<1>(t, b2)?, map.beta_tx::<0>(t, b2)?)
    };
    let f4 = {
        let d = map.beta_tx::<0>(t, volume as DartIdType)?;
        let b2 = map.beta_tx::<2>(t, d)?;
        (b2, map.beta_tx::<1>(t, b2)?, map.beta_tx::<0>(t, b2)?)
    };
    assert!(compute_tet_orientation(t, map, f1, p)? > 0.0);
    assert!(compute_tet_orientation(t, map, f2, p)? > 0.0);
    assert!(compute_tet_orientation(t, map, f3, p)? > 0.0);
    assert!(compute_tet_orientation(t, map, f4, p)? > 0.0);
    Ok(())
}
