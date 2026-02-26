use std::collections::VecDeque;

use coupe::nalgebra::Matrix5;
use honeycomb::{
    prelude::{CMap3, CoordsFloat, DartIdType, NULL_VOLUME_ID, Vertex3, VolumeIdType},
    stm::{Transaction, TransactionClosureResult, abort},
};
use rustc_hash::FxHashSet as HashSet;

use super::cavity::{Cavity3, CavityError};

const EPSILON: f64 = 1e-12;

#[derive(Debug, thiserror::Error)]
pub enum DelaunayError {
    #[error("point is located on a circumsphere")]
    CircumsphereSingularity,
    #[error("cavity building failed - {0}")]
    CavityBuilding(#[from] CavityError),
}

/// Compute the Delaunay cavity of a given point.
pub fn compute_delaunay_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    vol_id: VolumeIdType,
    p: Vertex3<T>,
) -> TransactionClosureResult<Cavity3<T>, DelaunayError> {
    let mut domain = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(vol_id);
    let mut marked = HashSet::default();
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

fn in_sphere<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    vol_id: VolumeIdType,
    p: &Vertex3<T>,
) -> TransactionClosureResult<bool, DelaunayError> {
    let [a, b, c, d] = [
        {
            let vid = map.vertex_id_tx(t, vol_id as DartIdType)?;
            if let Some(v) = map.read_vertex_tx(t, vid)? {
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
            if let Some(v) = map.read_vertex_tx(t, vid)? {
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
            if let Some(v) = map.read_vertex_tx(t, vid)? {
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
            if let Some(v) = map.read_vertex_tx(t, vid)? {
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

    let a = a.to_f64().unwrap();
    let b = b.to_f64().unwrap();
    let c = c.to_f64().unwrap();
    let d = d.to_f64().unwrap();
    let p = p.to_f64().unwrap();

    #[rustfmt::skip]
    let in_sphere = Matrix5::from_row_slice(&[
        a.x(), a.y(), a.z(), norm_squared(&a), 1.0,
        b.x(), b.y(), b.z(), norm_squared(&b), 1.0,
        c.x(), c.y(), c.z(), norm_squared(&c), 1.0,
        d.x(), d.y(), d.z(), norm_squared(&d), 1.0,
        p.x(), p.y(), p.z(), norm_squared(&p), 1.0,
    ]).determinant();

    if in_sphere.abs() <= EPSILON {
        abort(DelaunayError::CircumsphereSingularity)?;
    }

    Ok(in_sphere > 0.0)
}

#[inline]
fn norm_squared(v: &Vertex3<f64>) -> f64 {
    v.x() * v.x() + v.y() * v.y() + v.z() * v.z()
}
