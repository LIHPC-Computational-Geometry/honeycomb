use std::collections::{HashSet, VecDeque};

use honeycomb_core::{
    cmap::{CMap3, CMapBuilder, DartIdType, LinkError, NULL_VOLUME_ID, VolumeIdType},
    geometry::{CoordsFloat, Vertex3},
    stm::{
        Transaction, TransactionClosureResult, TransactionControl, TransactionResult, abort,
        atomically_with_err, try_or_coerce,
    },
};
use nalgebra::Matrix5;
use rand::{distr::Uniform, prelude::*};
use rayon::prelude::*;

use crate::{
    cavity::{Cavity3, carve_cavity_3d, extend_to_starshaped_cavity_3d, rebuild_cavity_3d},
    utils::{compute_tet_orientation, locate_containing_tet},
};

use super::cavity::CavityError;

const EPSILON: f64 = 1e-12;

#[derive(Debug, thiserror::Error)]
pub enum DelaunayError {
    #[error("point is located on a circumsphere")]
    CircumsphereSingularity,
    #[error("cavity building failed - {0}")]
    CavityBuilding(#[from] CavityError),
}

pub fn delaunay_box_3d<T: CoordsFloat>(lx: f64, ly: f64, lz: f64, n_points: usize) -> CMap3<T> {
    assert!(lx > 0.0);
    assert!(ly > 0.0);
    assert!(lz > 0.0);
    assert!(n_points > 0);

    // TODO: Sample points in the [0;lx]x[0;ly]x[0;lz] bounding box, build the actual box
    let points: Vec<Vertex3<T>> = sample_points(lx, ly, lz, n_points);
    let mut map = init_map(lx, ly, lz).expect("E: unreachable");

    map.allocate_unused_darts(n_points * 10 * 9);
    points.into_par_iter().for_each(|p| {
        let mut n_retry = 0;
        loop {
            match Transaction::with_control_and_err(
                |_| {
                    n_retry += 1;
                    TransactionControl::Retry
                },
                |t| {
                    // locate
                    // TODO: is 1 always valid as a starting point?

                    let volume = match locate_containing_tet(t, &map, 1, p)? {
                        Some(v) => v,
                        None => {
                            abort(DelaunayError::CavityBuilding(
                                CavityError::InconsistentState("..."),
                            ))?;
                            unreachable!();
                        }
                    };

                    #[cfg(debug_assertions)]
                    {
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
                    }

                    // compute cavity
                    let cavity = compute_delaunay_cavity_3d(t, &map, volume, p)?;
                    // carve
                    let carved_cavity =
                        try_or_coerce!(carve_cavity_3d(t, &map, cavity), DelaunayError);
                    // extend
                    let cavity = try_or_coerce!(
                        extend_to_starshaped_cavity_3d(t, &map, carved_cavity),
                        DelaunayError
                    );
                    // rebuild
                    try_or_coerce!(rebuild_cavity_3d(t, &map, cavity), DelaunayError);
                    Ok(())
                },
            ) {
                TransactionResult::Validated(_) => {
                    println!(
                        "insertion successful from t{:?}",
                        std::thread::current().id()
                    );
                    break;
                }
                TransactionResult::Abandoned => unreachable!(),
                TransactionResult::Cancelled(e) => {
                    eprintln!("E: insertion failed - {e}");
                    match e {
                        DelaunayError::CircumsphereSingularity => break,
                        DelaunayError::CavityBuilding(e) => match e {
                            CavityError::FailedOp(_)
                            | CavityError::FailedReservation(_)
                            | CavityError::InconsistentState(_) => {
                                n_retry += 1;
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
        println!("point processed after {n_retry} retries");
    });

    map
}

/// Compute the Delaunay cavity of a given point.
fn compute_delaunay_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    vol_id: VolumeIdType,
    p: Vertex3<T>,
) -> TransactionClosureResult<Cavity3<T>, DelaunayError> {
    let mut domain = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(vol_id);
    let mut marked = HashSet::new();
    marked.insert(NULL_VOLUME_ID);

    while let Some(vid) = queue.pop_front() {
        marked.insert(vid);
        let neighbors = [
            {
                let b3 = map.beta_tx::<3>(t, vid as DartIdType)?;
                map.volume_id_tx(t, b3)?
            },
            {
                let b2 = map.beta_tx::<2>(t, vid as DartIdType)?;
                let b3 = map.beta_tx::<3>(t, b2)?;
                map.volume_id_tx(t, b3)?
            },
            {
                let b0 = map.beta_tx::<0>(t, vid as DartIdType)?;
                let b2 = map.beta_tx::<2>(t, b0)?;
                let b3 = map.beta_tx::<3>(t, b2)?;
                map.volume_id_tx(t, b3)?
            },
            {
                let b1 = map.beta_tx::<1>(t, vid as DartIdType)?;
                let b2 = map.beta_tx::<2>(t, b1)?;
                let b3 = map.beta_tx::<3>(t, b2)?;
                map.volume_id_tx(t, b3)?
            },
        ];
        for vn in neighbors {
            if !marked.contains(&vn) && in_sphere(t, map, vn, &p)? {
                queue.push_back(vn);
            }
        }
        domain.push(vid);
    }

    Ok(Cavity3::new(p, domain))
}

// ref: https://arxiv.org/abs/1805.08831 section 2.4
fn in_sphere<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    vol_id: VolumeIdType,
    p: &Vertex3<T>,
) -> TransactionClosureResult<bool, DelaunayError> {
    let [a, b, c, d] = [
        {
            let vid = map.vertex_id_tx(t, vol_id as DartIdType)?;
            if let Some(v) = map.read_vertex(t, vid)? {
                v
            } else {
                return abort(DelaunayError::CavityBuilding(
                    CavityError::InconsistentState(
                        "topological vertex has no associated coordinates",
                    ),
                ))?;
            }
        },
        {
            let b1 = map.beta_tx::<1>(t, vol_id as DartIdType)?;
            let vid = map.vertex_id_tx(t, b1)?;
            if let Some(v) = map.read_vertex(t, vid)? {
                v
            } else {
                return abort(DelaunayError::CavityBuilding(
                    CavityError::InconsistentState(
                        "topological vertex has no associated coordinates",
                    ),
                ))?;
            }
        },
        {
            let b0 = map.beta_tx::<0>(t, vol_id as DartIdType)?;
            let vid = map.vertex_id_tx(t, b0)?;
            if let Some(v) = map.read_vertex(t, vid)? {
                v
            } else {
                return abort(DelaunayError::CavityBuilding(
                    CavityError::InconsistentState(
                        "topological vertex has no associated coordinates",
                    ),
                ))?;
            }
        },
        {
            let b2 = map.beta_tx::<2>(t, vol_id as DartIdType)?;
            let b0b2 = map.beta_tx::<0>(t, b2)?;
            let vid = map.vertex_id_tx(t, b0b2)?;
            if let Some(v) = map.read_vertex(t, vid)? {
                v
            } else {
                return abort(DelaunayError::CavityBuilding(
                    CavityError::InconsistentState(
                        "topological vertex has no associated coordinates",
                    ),
                ))?;
            }
        },
    ];

    #[rustfmt::skip]
    let in_sphere = Matrix5::from_row_slice(&[
        a.x().to_f64().unwrap(), a.y().to_f64().unwrap(), a.z().to_f64().unwrap(), norm_squared(&a), 1.0_f64,
        b.x().to_f64().unwrap(), b.y().to_f64().unwrap(), b.z().to_f64().unwrap(), norm_squared(&b), 1.0_f64,
        c.x().to_f64().unwrap(), c.y().to_f64().unwrap(), c.z().to_f64().unwrap(), norm_squared(&c), 1.0_f64,
        d.x().to_f64().unwrap(), d.y().to_f64().unwrap(), d.z().to_f64().unwrap(), norm_squared(&d), 1.0_f64,
        p.x().to_f64().unwrap(), p.y().to_f64().unwrap(), p.z().to_f64().unwrap(), norm_squared(&p), 1.0_f64,
    ]).determinant();

    if in_sphere.abs() <= EPSILON {
        abort(DelaunayError::CircumsphereSingularity)?;
    }

    Ok(in_sphere > 0.0)
}

#[inline]
fn norm_squared<T: CoordsFloat>(v: &Vertex3<T>) -> f64 {
    (v.x() * v.x() + v.y() * v.y() + v.z() * v.z())
        .to_f64()
        .unwrap()
}

fn sample_points<T: CoordsFloat>(lx: f64, ly: f64, lz: f64, n_points: usize) -> Vec<Vertex3<T>> {
    let mut rngx = SmallRng::seed_from_u64(123);
    let mut rngy = SmallRng::seed_from_u64(456);
    let mut rngz = SmallRng::seed_from_u64(789);
    let xs = {
        let dist = Uniform::try_from(0.0..lx).unwrap();
        dist.sample_iter(&mut rngx).take(n_points)
    };
    let ys = {
        let dist = Uniform::try_from(0.0..ly).unwrap();
        dist.sample_iter(&mut rngy).take(n_points)
    };
    let zs = {
        let dist = Uniform::try_from(0.0..lz).unwrap();
        dist.sample_iter(&mut rngz).take(n_points)
    };

    xs.zip(ys.zip(zs))
        .map(|(x, (y, z))| {
            let x = T::from(x).unwrap();
            let y = T::from(y).unwrap();
            let z = T::from(z).unwrap();
            Vertex3(x, y, z)
        })
        .collect()
}

// TODO: add image of our tet structure in doc
fn init_map<T: CoordsFloat>(lx: f64, ly: f64, lz: f64) -> Result<CMap3<T>, LinkError> {
    let map = CMapBuilder::<3, T>::from_n_darts(60)
        .build()
        .expect("E: unreachable");
    let zero = T::zero();
    let lx = T::from(lx).unwrap();
    let ly = T::from(ly).unwrap();
    let lz = T::from(lz).unwrap();

    if atomically_with_err(|t| {
        // tet 1
        map.link::<1>(t, 1, 2)?;
        map.link::<1>(t, 2, 3)?;
        map.link::<1>(t, 3, 1)?;
        map.link::<1>(t, 4, 5)?;
        map.link::<1>(t, 5, 6)?;
        map.link::<1>(t, 6, 4)?;
        map.link::<1>(t, 7, 8)?;
        map.link::<1>(t, 8, 9)?;
        map.link::<1>(t, 9, 7)?;
        map.link::<1>(t, 10, 11)?;
        map.link::<1>(t, 11, 12)?;
        map.link::<1>(t, 12, 10)?;

        map.link::<2>(t, 1, 4)?;
        map.link::<2>(t, 2, 7)?;
        map.link::<2>(t, 3, 10)?;
        map.link::<2>(t, 6, 8)?;
        map.link::<2>(t, 9, 11)?;
        map.link::<2>(t, 12, 5)?;

        // tet 2
        map.link::<1>(t, 13, 14)?;
        map.link::<1>(t, 14, 15)?;
        map.link::<1>(t, 15, 13)?;
        map.link::<1>(t, 16, 17)?;
        map.link::<1>(t, 17, 18)?;
        map.link::<1>(t, 18, 16)?;
        map.link::<1>(t, 19, 20)?;
        map.link::<1>(t, 20, 21)?;
        map.link::<1>(t, 21, 19)?;
        map.link::<1>(t, 22, 23)?;
        map.link::<1>(t, 23, 24)?;
        map.link::<1>(t, 24, 22)?;

        map.link::<2>(t, 13, 16)?;
        map.link::<2>(t, 14, 19)?;
        map.link::<2>(t, 15, 22)?;
        map.link::<2>(t, 18, 20)?;
        map.link::<2>(t, 21, 23)?;
        map.link::<2>(t, 24, 17)?;

        // tet 3
        map.link::<1>(t, 25, 26)?;
        map.link::<1>(t, 26, 27)?;
        map.link::<1>(t, 27, 25)?;
        map.link::<1>(t, 28, 29)?;
        map.link::<1>(t, 29, 30)?;
        map.link::<1>(t, 30, 28)?;
        map.link::<1>(t, 31, 32)?;
        map.link::<1>(t, 32, 33)?;
        map.link::<1>(t, 33, 31)?;
        map.link::<1>(t, 34, 35)?;
        map.link::<1>(t, 35, 36)?;
        map.link::<1>(t, 36, 34)?;

        map.link::<2>(t, 25, 28)?;
        map.link::<2>(t, 26, 31)?;
        map.link::<2>(t, 27, 34)?;
        map.link::<2>(t, 30, 32)?;
        map.link::<2>(t, 33, 35)?;
        map.link::<2>(t, 36, 29)?;

        // tet 4
        map.link::<1>(t, 37, 38)?;
        map.link::<1>(t, 38, 39)?;
        map.link::<1>(t, 39, 37)?;
        map.link::<1>(t, 40, 41)?;
        map.link::<1>(t, 41, 42)?;
        map.link::<1>(t, 42, 40)?;
        map.link::<1>(t, 43, 44)?;
        map.link::<1>(t, 44, 45)?;
        map.link::<1>(t, 45, 43)?;
        map.link::<1>(t, 46, 47)?;
        map.link::<1>(t, 47, 48)?;
        map.link::<1>(t, 48, 46)?;

        map.link::<2>(t, 37, 40)?;
        map.link::<2>(t, 38, 43)?;
        map.link::<2>(t, 39, 46)?;
        map.link::<2>(t, 42, 44)?;
        map.link::<2>(t, 45, 47)?;
        map.link::<2>(t, 48, 41)?;

        // tet 5
        map.link::<1>(t, 49, 50)?;
        map.link::<1>(t, 50, 51)?;
        map.link::<1>(t, 51, 49)?;
        map.link::<1>(t, 52, 53)?;
        map.link::<1>(t, 53, 54)?;
        map.link::<1>(t, 54, 52)?;
        map.link::<1>(t, 55, 56)?;
        map.link::<1>(t, 56, 57)?;
        map.link::<1>(t, 57, 55)?;
        map.link::<1>(t, 58, 59)?;
        map.link::<1>(t, 59, 60)?;
        map.link::<1>(t, 60, 58)?;

        map.link::<2>(t, 49, 52)?;
        map.link::<2>(t, 50, 55)?;
        map.link::<2>(t, 51, 58)?;
        map.link::<2>(t, 54, 56)?;
        map.link::<2>(t, 57, 59)?;
        map.link::<2>(t, 60, 53)?;

        // link all tetrahedra together
        map.link::<3>(t, 7, 49)?;
        map.link::<3>(t, 19, 59)?;
        map.link::<3>(t, 31, 52)?;
        map.link::<3>(t, 43, 57)?;

        // set vertices
        map.write_vertex(t, 1, Vertex3(zero, zero, zero))?;
        map.write_vertex(t, 2, Vertex3(lx, zero, zero))?;
        map.write_vertex(t, 3, Vertex3(zero, zero, lz))?;
        map.write_vertex(t, 6, Vertex3(zero, ly, zero))?;
        map.write_vertex(t, 13, Vertex3(lx, ly, zero))?;
        map.write_vertex(t, 15, Vertex3(lx, ly, lz))?;
        map.write_vertex(t, 25, Vertex3(lx, zero, lz))?;
        map.write_vertex(t, 37, Vertex3(zero, ly, lz))?;

        Ok(())
    })
    .is_err()
    {
        unreachable!();
    }

    Ok(map)
}
