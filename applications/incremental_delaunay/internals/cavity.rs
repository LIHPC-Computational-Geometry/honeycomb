use std::cell::RefCell;

use honeycomb::{
    core::{
        cmap::{
            CMap3, DartIdType, DartReleaseError, DartReservationError, FaceIdType, LinkError,
            NULL_DART_ID, OrbitPolicy, SewError, VolumeIdType,
        },
        geometry::{CoordsFloat, Vertex3},
        stm::{
            StmClosureResult, Transaction, TransactionClosureResult, abort, retry, try_or_coerce,
        },
    },
    stm::{TVar, TransactionError},
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use smallvec::{SmallVec, smallvec};

use super::compute_tet_orientation;

thread_local! {
    pub static DART_BLOCK_START: RefCell<TVar<DartIdType>> = RefCell::new(TVar::new(1));
}

type CavityBoundary3 = HashMap<FaceIdType, [(DartIdType, DartIdType); 3]>;
type CavityInternal3 = HashSet<FaceIdType>;

pub struct Cavity3<T: CoordsFloat> {
    point: Vertex3<T>,
    domain: Vec<VolumeIdType>,
}

impl<T: CoordsFloat> Cavity3<T> {
    /// Constructor.
    pub fn new(point: Vertex3<T>, domain: Vec<VolumeIdType>) -> Self {
        Self { point, domain }
    }
}

pub struct CarvedCavity3<T: CoordsFloat> {
    point: Vertex3<T>,
    boundary: CavityBoundary3,
    free_darts: Vec<DartIdType>,
}

#[derive(Debug, thiserror::Error)]
pub enum CavityError {
    #[error("core operation failed: {0}")]
    FailedOp(#[from] SewError),
    #[error("dart release failed: {0}")]
    FailedRelease(#[from] DartReleaseError),
    #[error("dart release failed: {0}")]
    FailedReservation(#[from] DartReservationError),
    #[error("mesh is in an inconsistent state: {0}")]
    InconsistentState(&'static str),
    #[error("cannot extend the cavity: {0}")]
    NonExtendable(&'static str),
}

impl From<LinkError> for CavityError {
    fn from(value: LinkError) -> Self {
        Self::FailedOp(SewError::FailedLink(value))
    }
}

// -- cavity computation

/// Extend a cavity until it can be triangulated from its point.
pub fn extend_to_starshaped_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: CarvedCavity3<T>,
) -> TransactionClosureResult<CarvedCavity3<T>, CavityError> {
    let CarvedCavity3 {
        point,
        mut boundary,
        mut free_darts,
    } = cavity;

    // d1 = d, d2 = b1(d), d3 = b0(d)
    'outer: loop {
        for (f, [(d1, d1_neigh), (d2, d2_neigh), (d3, d3_neigh)]) in &boundary {
            if compute_tet_orientation(t, map, (*d1, *d2, *d3), cavity.point)? < 0.0 {
                let mut to_remove: SmallVec<_, 4> = smallvec!(*f); // technically could be 3-long
                let mut to_add: SmallVec<_, 4> = smallvec!(); // same
                let face_to_check = [
                    {
                        let d = map.beta_tx::<3>(t, *d1)?;
                        if d == NULL_DART_ID {
                            abort(CavityError::NonExtendable(
                                "cannot create a star-shaped cavity",
                            ))?;
                        }
                        (map.beta_tx::<2>(t, d)?, d1_neigh)
                    },
                    {
                        let d = map.beta_tx::<3>(t, *d2)?;
                        (map.beta_tx::<2>(t, d)?, d2_neigh)
                    },
                    {
                        let d = map.beta_tx::<3>(t, *d3)?;
                        (map.beta_tx::<2>(t, d)?, d3_neigh)
                    },
                ];
                for (da, d_neigh) in face_to_check {
                    let fid_a = map.face_id_tx(t, da)?;
                    if boundary.contains_key(&fid_a) {
                        to_remove.push(fid_a);
                    } else {
                        let db = map.beta_tx::<1>(t, da)?;
                        let dc = map.beta_tx::<0>(t, da)?;

                        let adjacency = [
                            (da, *d_neigh),
                            (db, {
                                let b2 = map.beta_tx::<2>(t, db)?;
                                let fid_b = map.face_id_tx(t, b2)?;
                                if let Some(adj) = boundary.get(&fid_b) {
                                    adj.iter()
                                        .find_map(|(dd, dd_neigh)| {
                                            if map.beta_tx::<3>(t, *dd).ok()? == b2 {
                                                Some(*dd_neigh)
                                            } else {
                                                None
                                            }
                                        })
                                        .ok_or(TransactionError::Abort(
                                            CavityError::InconsistentState("adjacency out-of-date"),
                                        ))?
                                } else {
                                    b2
                                }
                            }),
                            (dc, {
                                let b2 = map.beta_tx::<2>(t, dc)?;
                                let fid_c = map.face_id_tx(t, b2)?;
                                if let Some(adj) = boundary.get(&fid_c) {
                                    adj.iter()
                                        .find_map(|(dd, dd_neigh)| {
                                            if map.beta_tx::<3>(t, *dd).ok()? == b2 {
                                                Some(*dd_neigh)
                                            } else {
                                                None
                                            }
                                        })
                                        .ok_or(TransactionError::Abort(
                                            CavityError::InconsistentState("adjacency out-of-date"),
                                        ))?
                                } else {
                                    b2
                                }
                            }),
                        ];
                        to_add.push((fid_a, adjacency));
                    }
                }

                let mut buffer: SmallVec<_, 6> = SmallVec::new();
                for f in to_remove {
                    for d in map.orbit_tx(t, OrbitPolicy::Face, f) {
                        buffer.push(d?);
                    }
                    for &d in &buffer {
                        try_or_coerce!(map.unsew_tx::<1>(t, d), CavityError);
                        if map.beta_tx::<2>(t, d)? != NULL_DART_ID {
                            try_or_coerce!(map.unsew_tx::<2>(t, d), CavityError);
                        }
                        if map.beta_tx::<3>(t, d)? != NULL_DART_ID {
                            try_or_coerce!(map.unsew_tx::<3>(t, d), CavityError);
                        }
                    }
                    for d in buffer.drain(..) {
                        try_or_coerce!(map.release_dart_tx(t, d), CavityError);
                        free_darts.push(d);
                    }
                }

                for (f, val) in to_add {
                    for (d, d_neigh) in val {
                        if let Some(val) = boundary.get_mut(&map.face_id_tx(t, d_neigh)?)
                            && let Some(pair) = val.iter_mut().find(|(dd, _)| *dd == d_neigh)
                        {
                            pair.1 = d;
                        } else {
                            abort(CavityError::InconsistentState(
                                "found a neighbor face without dart adjacency data",
                            ))?;
                        }
                    }
                    boundary.insert(f, val);
                }
                continue 'outer;
            }
        }

        break;
    }

    Ok(CarvedCavity3 {
        point,
        boundary,
        free_darts,
    })
}

/// Compute data representations for a cavity's boundary and internal elements.
pub fn map_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: &Cavity3<T>,
) -> StmClosureResult<(CavityBoundary3, CavityInternal3)> {
    let mut boundary_faces: HashMap<FaceIdType, [DartIdType; 3]> = HashMap::default();
    let mut cavity_internals = CavityInternal3::default();
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
                    [fd, map.beta_tx::<1>(t, fd)?, map.beta_tx::<0>(t, fd)?],
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
                    [fd, map.beta_tx::<1>(t, fd)?, map.beta_tx::<0>(t, fd)?],
                );
            }
        }
    }

    // build the cavity adjacency graph

    let mut cavity_map = CavityBoundary3::default();
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
                && d != d1
            {
                d1_neighbor = d;
                break; // rest of the buffer is still emptied by `drain`
            }
        }
        if d1_neighbor == NULL_DART_ID {
            retry()?;
        }

        for d in map.orbit_tx(t, OrbitPolicy::Edge, d2) {
            buffer.push(d?);
        }
        for d in buffer.drain(..) {
            if boundary_faces.contains_key(&map.face_id_tx(t, d)?)
                && cavity.domain.contains(&map.volume_id_tx(t, d)?)
                && d != d2
            {
                d2_neighbor = d;
                break;
            }
        }
        if d2_neighbor == NULL_DART_ID {
            retry()?;
        }

        for d in map.orbit_tx(t, OrbitPolicy::Edge, d3) {
            buffer.push(d?);
        }
        for d in buffer.drain(..) {
            if boundary_faces.contains_key(&map.face_id_tx(t, d)?)
                && cavity.domain.contains(&map.volume_id_tx(t, d)?)
                && d != d3
            {
                d3_neighbor = d;
                break;
            }
        }
        if d3_neighbor == NULL_DART_ID {
            retry()?;
        }

        cavity_map.insert(f, [(d1, d1_neighbor), (d2, d2_neighbor), (d3, d3_neighbor)]);
    }

    Ok((cavity_map, cavity_internals))
}

// cavity modification

/// Delete all internal elements of a cavity.
pub fn carve_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: Cavity3<T>,
) -> TransactionClosureResult<CarvedCavity3<T>, CavityError> {
    let (cavity_map, cavity_internals) = map_cavity_3d(t, map, &cavity)?;
    let mut free_darts = Vec::new();
    let mut buffer = Vec::with_capacity(6);

    for f in cavity_internals {
        for d in map.orbit_tx(t, OrbitPolicy::Face, f) {
            buffer.push(d?);
        }
        for &d in &buffer {
            if map.beta_tx::<3>(t, d)? != NULL_DART_ID {
                try_or_coerce!(map.unsew_tx::<3>(t, d), CavityError);
            }
        }
        for &d in &buffer {
            if map.beta_tx::<2>(t, d)? != NULL_DART_ID {
                try_or_coerce!(map.unsew_tx::<2>(t, d), CavityError);
            }
        }
        for &d in &buffer {
            try_or_coerce!(map.unlink_tx::<1>(t, d), CavityError);
        }
        for d in buffer.drain(..) {
            try_or_coerce!(map.release_dart_tx(t, d), CavityError);
            free_darts.push(d);
        }
    }

    // necessary for tets that had two or more faces adjacent to the boundary
    for &[(d1, _), (d2, _), (d3, _)] in cavity_map.values() {
        if map.beta_tx::<2>(t, d1)? != NULL_DART_ID {
            try_or_coerce!(map.unsew_tx::<2>(t, d1), CavityError);
        }
        if map.beta_tx::<2>(t, d2)? != NULL_DART_ID {
            try_or_coerce!(map.unsew_tx::<2>(t, d2), CavityError);
        }
        if map.beta_tx::<2>(t, d3)? != NULL_DART_ID {
            try_or_coerce!(map.unsew_tx::<2>(t, d3), CavityError);
        }
    }

    Ok(CarvedCavity3 {
        point: cavity.point,
        boundary: cavity_map,
        free_darts,
    })
}

pub fn rebuild_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: CarvedCavity3<T>,
) -> TransactionClosureResult<VolumeIdType, CavityError> {
    let CarvedCavity3 {
        point,
        boundary,
        mut free_darts,
    } = cavity;
    let n_required_darts = boundary.len() * 9;
    free_darts.truncate(n_required_darts);
    for d in &free_darts {
        map.claim_dart_tx(t, *d)?;
    }
    if free_darts.len() < n_required_darts {
        let start = DART_BLOCK_START.with_borrow(|r| r.read(t))?;
        // this method returns claimed darts
        let mut new_darts: Vec<DartIdType> = try_or_coerce!(
            map.reserve_darts_from_tx(t, n_required_darts - free_darts.len(), start),
            CavityError
        );
        DART_BLOCK_START.with_borrow(|r| r.modify(t, |v| v + new_darts.len() as DartIdType))?;
        free_darts.append(&mut new_darts);
    }
    free_darts.sort(); // TODO: figure out why this is needed to keep a valid structure

    let tmp = free_darts[1];

    assert_eq!(free_darts.len() % 9, 0);
    debug_assert!(
        free_darts
            .iter()
            .all(|&d| { free_darts.iter().filter(|&&dd| d == dd).count() == 1 })
    );

    for ((_, [(da, da_neigh), (db, db_neigh), (dc, dc_neigh)]), nds) in
        boundary.into_iter().zip(free_darts.chunks_exact(9))
    {
        let nds @ [_d1, d2, _, _, d5, _, _, d8, _]: [DartIdType; 9] =
            nds.try_into().expect("E: unreachable");
        try_or_coerce!(make_incomplete_tet(t, map, nds, point), CavityError);

        try_or_coerce!(map.sew_tx::<2>(t, d2, db), CavityError);
        let b2db = map.beta_tx::<2>(t, db_neigh)?;
        if b2db != NULL_DART_ID {
            try_or_coerce!(map.sew_tx::<3>(t, b2db, d2), CavityError);
        }

        try_or_coerce!(map.sew_tx::<2>(t, d5, da), CavityError);
        let b2da = map.beta_tx::<2>(t, da_neigh)?;
        if b2da != NULL_DART_ID {
            try_or_coerce!(map.sew_tx::<3>(t, b2da, d5), CavityError);
        }

        try_or_coerce!(map.sew_tx::<2>(t, d8, dc), CavityError);
        let b2dc = map.beta_tx::<2>(t, dc_neigh)?;
        if b2dc != NULL_DART_ID {
            try_or_coerce!(map.sew_tx::<3>(t, b2dc, d8), CavityError);
        }
    }

    Ok(map.volume_id_tx(t, tmp)?)
}

fn make_incomplete_tet<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    [d1, d2, d3, d4, d5, d6, d7, d8, d9]: [DartIdType; 9],
    p: Vertex3<T>,
) -> TransactionClosureResult<(), LinkError> {
    // build 3 triangles
    map.link_tx::<1>(t, d1, d2)?;
    map.link_tx::<1>(t, d2, d3)?;
    map.link_tx::<1>(t, d3, d1)?;
    map.link_tx::<1>(t, d4, d5)?;
    map.link_tx::<1>(t, d5, d6)?;
    map.link_tx::<1>(t, d6, d4)?;
    map.link_tx::<1>(t, d7, d8)?;
    map.link_tx::<1>(t, d8, d9)?;
    map.link_tx::<1>(t, d9, d7)?;
    // link the sides that will be adjacent to the point inserted by the cavity
    map.link_tx::<2>(t, d3, d4)?;
    map.link_tx::<2>(t, d6, d7)?;
    map.link_tx::<2>(t, d9, d1)?;

    let vid = map.vertex_id_tx(t, d1)?;
    map.write_vertex_tx(t, vid, p)?;

    Ok(())
}
