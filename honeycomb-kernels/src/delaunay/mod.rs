use std::collections::{HashSet, VecDeque};

use honeycomb_core::{
    cmap::{CMap3, CMapBuilder, DartIdType, NULL_VOLUME_ID, VolumeIdType},
    geometry::{CoordsFloat, Vertex3},
    stm::{
        Transaction, TransactionClosureResult, abort, atomically_with_err, retry, try_or_coerce,
    },
};
use nalgebra::Matrix5;

use crate::{
    cavity::{Cavity3, carve_cavity_3d, extend_to_starshaped_cavity_3d, rebuild_cavity_3d},
    utils::locate_containing_tet,
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
    let points: Vec<Vertex3<T>> = Vec::with_capacity(n_points);
    let mut map = CMapBuilder::<3, T>::from_n_darts(60)
        .build()
        .expect("E: unreachable");

    map.allocate_unused_darts(n_points * 10 * 9);
    points.into_iter().for_each(|p| {
        while let Err(e) = atomically_with_err(|t| {
            // locate
            // TODO: is 1 always valid as a starting point?
            let volume = match locate_containing_tet(t, &map, 1, p)? {
                Some(v) => v,
                None => {
                    return retry()?; // can only happen due to parallel inconsistency here
                }
            };
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
            match e {
                DelaunayError::CircumsphereSingularity => break,
                DelaunayError::CavityBuilding(e) => match e {
                    CavityError::FailedOp(_) | CavityError::FailedReservation(_) => continue,
                    CavityError::FailedRelease(_) | CavityError::NonExtendable(_) => break,
                },
            }
        }
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
            if !marked.contains(&vn) {
                if in_sphere(t, map, vn, &p)? {
                    queue.push_back(vn);
                }
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
            map.read_vertex(t, vid)?.expect("E: unreachable")
        },
        {
            let b1 = map.beta_tx::<1>(t, vol_id as DartIdType)?;
            let vid = map.vertex_id_tx(t, b1)?;
            map.read_vertex(t, vid)?.expect("E: unreachable")
        },
        {
            let b0 = map.beta_tx::<0>(t, vol_id as DartIdType)?;
            let vid = map.vertex_id_tx(t, b0)?;
            map.read_vertex(t, vid)?.expect("E: unreachable")
        },
        {
            let b2 = map.beta_tx::<2>(t, vol_id as DartIdType)?;
            let b0b2 = map.beta_tx::<3>(t, b2)?;
            let vid = map.vertex_id_tx(t, b0b2)?;
            map.read_vertex(t, vid)?.expect("E: unreachable")
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
