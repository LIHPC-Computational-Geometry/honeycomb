//! implementation of the cavity operator

use std::collections::{HashMap, HashSet};

use honeycomb_core::{
    cmap::{
        CMap3, DartIdType, DartReleaseError, FaceIdType, NULL_DART_ID, OrbitPolicy, SewError,
        VolumeIdType,
    },
    geometry::{CoordsFloat, Vertex3},
    stm::{StmClosureResult, Transaction, TransactionClosureResult, try_or_coerce},
};

pub struct Cavity3<T: CoordsFloat> {
    point: Vertex3<T>,
    domain: Vec<VolumeIdType>,
    // n_internal_faces: usize,
}

pub type CavityBoundary3 = HashMap<FaceIdType, [(DartIdType, DartIdType); 3]>;
pub type CavityInternal3 = HashSet<FaceIdType>;

#[derive(Debug, thiserror::Error)]
pub enum CavityError {
    #[error("core operation failed: {0}")]
    OpError(#[from] SewError),
    #[error("dart release failed: {0}")]
    DartReleaseError(#[from] DartReleaseError),
}

// -- cavity computation

pub fn compute_delaunay_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    vid: VolumeIdType,
    p: Vertex3<T>,
) -> TransactionClosureResult<Cavity3<T>, CavityError> {
    todo!()
}

pub fn reduce_to_starshaped_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: Cavity3<T>,
) -> TransactionClosureResult<Cavity3<T>, CavityError> {
    todo!()
}

pub fn extend_to_starshaped_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: Cavity3<T>,
) -> TransactionClosureResult<Cavity3<T>, CavityError> {
    todo!()
}

/// Compute data representations for the cavity's boundary and internal elements.
pub fn map_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: &Cavity3<T>,
) -> StmClosureResult<(CavityBoundary3, CavityInternal3)> {
    let mut boundary_faces: HashMap<FaceIdType, [DartIdType; 3]> =
        HashMap::with_capacity(cavity.domain.len());
    let mut cavity_internals = CavityInternal3::with_capacity(cavity.domain.len());
    for &vol in &cavity.domain {
        let d = vol as DartIdType;
        let b0d = map.beta_tx::<0>(t, d)?;
        let b1d = map.beta_tx::<1>(t, d)?;
        let face_darts = [
            d,
            map.beta_tx::<2>(t, d)?,
            map.beta_tx::<2>(t, b0d)?,
            map.beta_tx::<2>(t, b1d)?,
        ];
        for fd in face_darts {
            let b3 = map.beta_tx::<3>(t, fd)?;
            if b3 == NULL_DART_ID {
                // boundary of the cavity and of the mesh
                boundary_faces.insert(
                    map.face_id_tx(t, fd)?,
                    [fd, map.beta_tx::<0>(t, fd)?, map.beta_tx::<1>(t, fd)?],
                );
                continue;
            }
            let adj_vol = map.volume_id_tx(t, b3)?;
            if cavity.domain.contains(&adj_vol) {
                // internal face
                cavity_internals.insert(map.face_id_tx(t, fd)?);
            } else {
                // boundary face
                boundary_faces.insert(
                    map.face_id_tx(t, fd)?,
                    [fd, map.beta_tx::<0>(t, fd)?, map.beta_tx::<1>(t, fd)?],
                );
            }
        }
    }

    // build the cavity adjacency graph

    let mut cavity_map = CavityBoundary3::with_capacity(boundary_faces.len());
    let mut buffer = Vec::with_capacity(16);
    for (&f, &[d1, d2, d3]) in boundary_faces.iter() {
        let mut d1_neighbor = NULL_DART_ID;
        let mut d2_neighbor = NULL_DART_ID;
        let mut d3_neighbor = NULL_DART_ID;

        for d in map.orbit_tx(t, OrbitPolicy::Edge, d1) {
            buffer.push(d?);
        }
        for d in buffer.drain(..) {
            if boundary_faces.contains_key(&map.face_id_tx(t, d)?)
                && cavity.domain.contains(&map.volume_id_tx(t, d)?)
            {
                d1_neighbor = d;
                break; // rest of the buffer is still emptied by `drain`
            }
        }

        for d in map.orbit_tx(t, OrbitPolicy::Edge, d2) {
            buffer.push(d?);
        }
        for d in buffer.drain(..) {
            if boundary_faces.contains_key(&map.face_id_tx(t, d)?)
                && cavity.domain.contains(&map.volume_id_tx(t, d)?)
            {
                d2_neighbor = d;
                break;
            }
        }

        for d in map.orbit_tx(t, OrbitPolicy::Edge, d3) {
            buffer.push(d?);
        }
        for d in buffer.drain(..) {
            if boundary_faces.contains_key(&map.face_id_tx(t, d)?)
                && cavity.domain.contains(&map.volume_id_tx(t, d)?)
            {
                d3_neighbor = d;
                break;
            }
        }

        // FIXME: use an error instead of assertions
        assert_ne!(d1_neighbor, NULL_DART_ID);
        assert_ne!(d2_neighbor, NULL_DART_ID);
        assert_ne!(d3_neighbor, NULL_DART_ID);

        cavity_map.insert(f, [(d1, d1_neighbor), (d2, d2_neighbor), (d3, d3_neighbor)]);
    }

    Ok((cavity_map, cavity_internals))
}

// cavity modification

pub fn carve_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: &Cavity3<T>,
) -> TransactionClosureResult<CavityBoundary3, CavityError> {
    let (cavity_map, cavity_internals) = map_cavity_3d(t, map, cavity)?;
    let mut buffer = Vec::with_capacity(16);

    for f in cavity_internals {
        for d in map.orbit_tx(t, OrbitPolicy::Face, f) {
            buffer.push(d?);
        }
        for &d in &buffer {
            try_or_coerce!(map.unsew::<1>(t, d), CavityError);
            if map.beta_tx::<2>(t, d)? != NULL_DART_ID {
                try_or_coerce!(map.unsew::<2>(t, d), CavityError);
            }
            if map.beta_tx::<3>(t, d)? != NULL_DART_ID {
                try_or_coerce!(map.unsew::<3>(t, d), CavityError);
            }
        }
        for d in buffer.drain(..) {
            try_or_coerce!(map.release_dart_tx(t, d), CavityError);
        }
    }

    Ok(cavity_map)
}
