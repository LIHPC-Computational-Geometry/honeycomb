mod cavity;
mod delaunay;

use std::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use coupe::{Partition, Point3D, ZCurve, nalgebra::Matrix3};
use honeycomb::{
    core::{
        cmap::{CMap3, CMapBuilder, DartIdType, VolumeIdType},
        geometry::{CoordsFloat, Vertex3},
        stm::{StmError, Transaction, abort, atomically_with_err, try_or_coerce},
    },
    prelude::{NULL_DART_ID, grid_generation::GridBuilder},
    stm::{StmClosureResult, retry},
};
use rand::{distr::Uniform, prelude::*};
use rayon::prelude::*;

use cavity::{CavityError, carve_cavity_3d, extend_to_starshaped_cavity_3d, rebuild_cavity_3d};
use delaunay::{DelaunayError, compute_delaunay_cavity_3d};

thread_local! {
    pub static LAST_INSERTED: Cell<VolumeIdType> = const { Cell::new(1) };
}

pub fn delaunay_box_3d<T: CoordsFloat>(
    lx: f64,
    ly: f64,
    lz: f64,
    n_points: usize,
    n_points_init: usize,
    file_init: Option<String>,
    seed: u64,
) -> CMap3<T> {
    assert!(lx > 0.0);
    assert!(ly > 0.0);
    assert!(lz > 0.0);
    assert!(n_points > 0);

    println!("| Delaunay box triangulation benchmark");
    println!("|-> sampling domain: [0;{lx}]x[0;{ly}]x[0;{lz}]");
    println!("|-> seq. inserts   : {n_points_init}");
    println!("|-> par. inserts   : {n_points}");
    println!(
        "|-> threads used   : {}",
        std::env::var("RAYON_NUM_THREADS")
            .ok()
            .map(|s| s.parse().ok())
            .flatten()
            .unwrap_or(1)
    );

    let mut instant = Instant::now();
    let mut all_points: Vec<Vertex3<T>> = sample_points(lx, ly, lz, n_points_init + n_points, seed);
    let time = instant.elapsed().as_secs_f32();
    println!(
        " sample | {:>8} | {:>8.3e} |",
        n_points_init + n_points,
        time,
    );

    instant = Instant::now();
    let points_init: Vec<Vertex3<T>> = {
        let tmp: Vec<_> = all_points.drain(..n_points_init).collect();
        let ps: Vec<_> = tmp
            .iter()
            .map(|v| {
                let v = v.to_f64().unwrap();
                Point3D::new(v.0, v.1, v.2)
            })
            .collect();
        let mut partition = vec![0; tmp.len()];
        ZCurve {
            part_count: 4,
            order: 2,
        }
        .partition(&mut partition, &ps)
        .unwrap();

        let mut tmp: Vec<_> = tmp.into_iter().zip(partition.into_iter()).collect();
        tmp.sort_by(|(_, p_a), (_, p_b)| p_a.cmp(p_b));

        tmp.into_iter().map(|v| v.0).collect()
    };

    let points: Vec<Vertex3<T>> = {
        let ps: Vec<_> = all_points
            .iter()
            .map(|v| {
                let v = v.to_f64().unwrap();
                Point3D::new(v.0, v.1, v.2)
            })
            .collect();
        let mut partition = vec![0; all_points.len()];
        ZCurve {
            part_count: 4,
            order: 5, // TODO: compute according to n_threads
        }
        .partition(&mut partition, &ps)
        .unwrap();

        let mut tmp: Vec<_> = all_points.into_iter().zip(partition.into_iter()).collect();
        tmp.sort_by(|(_, p_a), (_, p_b)| p_a.cmp(p_b));

        tmp.into_iter().map(|v| v.0).collect()
    };
    let time = instant.elapsed().as_secs_f32();
    println!(
        " sort   | {:>8} | {:>8.3e} |",
        n_points_init + n_points,
        time,
    );
    let mut map = if let Some(f) = file_init {
        CMapBuilder::<3>::from_cmap_file(f.as_str())
            .build()
            .expect("E: bad input file")
    } else {
        GridBuilder::<3, _>::default()
            .n_cells([1, 1, 1])
            .len_per_cell([
                T::from(lx).unwrap(),
                T::from(ly).unwrap(),
                T::from(lz).unwrap(),
            ])
            .split_cells(true)
            .build()
            .unwrap()
    };

    // typical point distribution will result in 6-8*n_points tets
    // 20 gives some leeway, 12 is the number of darts per tet
    // when init from file, there may be already unused darts
    let n_unused = map.n_unused_darts();
    map.allocate_unused_darts(20 * 12 * (n_points_init + n_points) - n_unused);

    instant = Instant::now();
    let mut count = 0;
    points_init.into_iter().for_each(|p| {
        loop {
            match atomically_with_err(|t| {
                // locate
                let res = locate_containing_tet(t, &map, LAST_INSERTED.get(), p);
                if let Err(StmError::Failure) = res {
                    abort(DelaunayError::CavityBuilding(
                        CavityError::InconsistentState("..."),
                    ))?;
                };
                let volume = match res? {
                    Some(v) => v,
                    None => {
                        abort(DelaunayError::CavityBuilding(
                            CavityError::InconsistentState("..."),
                        ))?;
                        unreachable!();
                    }
                };

                #[cfg(debug_assertions)]
                check_tet_orientation(t, &map, volume, p)?;

                // compute cavity
                let cavity = compute_delaunay_cavity_3d(t, &map, volume, p)?;
                // carve
                let carved_cavity = try_or_coerce!(carve_cavity_3d(t, &map, cavity), DelaunayError);
                // extend
                let cavity = try_or_coerce!(
                    extend_to_starshaped_cavity_3d(t, &map, carved_cavity),
                    DelaunayError
                );
                // rebuild
                try_or_coerce!(rebuild_cavity_3d(t, &map, cavity), DelaunayError);
                Ok(())
            }) {
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
        " init   | {count:>8} | {:>8.3e} | {:>8.3e}",
        time,
        count as f32 / time,
    );

    instant = Instant::now();
    let num_threads = rayon::current_num_threads();
    let counters: Vec<AtomicUsize> = (0..num_threads).map(|_| AtomicUsize::new(0)).collect();
    points.into_par_iter().for_each(|p| {
        // points.par_chunks(128).for_each(|c| {
        // c.into_iter().for_each(|&p| {
        loop {
            match atomically_with_err(|t| {
                // locate
                let res = locate_containing_tet(t, &map, 1, p);
                if let Err(StmError::Failure) = res {
                    abort(DelaunayError::CavityBuilding(
                        CavityError::InconsistentState("..."),
                    ))?;
                };
                let volume = match res? {
                    Some(v) => v,
                    None => {
                        abort(DelaunayError::CavityBuilding(
                            CavityError::InconsistentState("..."),
                        ))?;
                        unreachable!();
                    }
                };

                #[cfg(debug_assertions)]
                check_tet_orientation(t, &map, volume, p)?;

                // compute cavity
                let cavity = compute_delaunay_cavity_3d(t, &map, volume, p)?;
                // carve
                let carved_cavity = try_or_coerce!(carve_cavity_3d(t, &map, cavity), DelaunayError);
                // extend
                let cavity = try_or_coerce!(
                    extend_to_starshaped_cavity_3d(t, &map, carved_cavity),
                    DelaunayError
                );
                // rebuild
                try_or_coerce!(rebuild_cavity_3d(t, &map, cavity), DelaunayError);
                Ok(())
            }) {
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
                            CavityError::FailedRelease(_) | CavityError::NonExtendable(_) => {
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
        " insert | {count:>8} | {:>8.3e} | {:>8.3e}",
        time,
        count as f32 / time,
    );

    map
}

#[rustfmt::skip]
pub fn compute_tet_orientation<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    (d1, d2, d3): (DartIdType, DartIdType, DartIdType),
    p: Vertex3<T>,
) -> StmClosureResult<f64> {
    let v1 = {
        let vid = map.vertex_id_tx(t, d1)?;
        if let Some(v) = map.read_vertex(t, vid)? {
            v
        } else {
            return retry()?;
        }
    };
    let v2 = {
        let vid = map.vertex_id_tx(t, d2)?;
        if let Some(v) = map.read_vertex(t, vid)? {
            v
        } else {
            return retry()?;
        }
    };
    let v3 = {
        let vid = map.vertex_id_tx(t, d3)?;
        if let Some(v) = map.read_vertex(t, vid)? {
            v
        } else {
            return retry()?;
        }
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

fn locate_containing_tet<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    start: VolumeIdType,
    p: Vertex3<T>,
) -> StmClosureResult<Option<VolumeIdType>> {
    fn locate_next_tet<T: CoordsFloat>(
        t: &mut Transaction,
        map: &CMap3<T>,
        d: DartIdType,
        p: Vertex3<T>,
    ) -> StmClosureResult<Option<DartIdType>> {
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

    let mut count = 0;
    let max_walk = map.n_darts() / 12;
    let mut dart = start as DartIdType;

    loop {
        count += 1;
        if count > max_walk {
            Err(StmError::Failure)?;
        }
        if let Some(next_dart) = locate_next_tet(t, map, dart, p)? {
            dart = next_dart;
            // point is outside or across a gap in the mesh
            if dart == NULL_DART_ID {
                // it is possible to look for another path, but it requires a more complex condition
                // than "just follow the first neg volume direction"
                return Ok(None);
            }
        } else {
            return Ok(Some(map.volume_id_tx(t, dart)?));
        }
    }
}

fn sample_points<T: CoordsFloat>(
    lx: f64,
    ly: f64,
    lz: f64,
    n_points: usize,
    seed: u64,
) -> Vec<Vertex3<T>> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let xs: Vec<_> = {
        let dist = Uniform::try_from(0.0..lx).unwrap();
        dist.sample_iter(&mut rng).take(n_points).collect()
    };
    let ys: Vec<_> = {
        let dist = Uniform::try_from(0.0..ly).unwrap();
        dist.sample_iter(&mut rng).take(n_points).collect()
    };
    let zs: Vec<_> = {
        let dist = Uniform::try_from(0.0..lz).unwrap();
        dist.sample_iter(&mut rng).take(n_points).collect()
    };

    xs.into_iter()
        .zip(ys.into_iter().zip(zs.into_iter()))
        .map(|(x, (y, z))| {
            let x = T::from(x).unwrap();
            let y = T::from(y).unwrap();
            let z = T::from(z).unwrap();
            Vertex3(x, y, z)
        })
        .collect()
}

#[cfg(debug_assertions)]
fn check_tet_orientation<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    volume: VolumeIdType,
    p: Vertex3<T>,
) -> StmClosureResult<()> {
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
    assert!(compute_tet_orientation(t, &map, f1, p)? > 0.0);
    assert!(compute_tet_orientation(t, &map, f2, p)? > 0.0);
    assert!(compute_tet_orientation(t, &map, f3, p)? > 0.0);
    assert!(compute_tet_orientation(t, &map, f4, p)? > 0.0);
    Ok(())
}
