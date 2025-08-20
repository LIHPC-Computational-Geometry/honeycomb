//! implementation of the cavity operator

use std::collections::{HashMap, HashSet};

use honeycomb_core::{
    cmap::{
        CMap3, DartIdType, DartReleaseError, DartReservationError, FaceIdType, NULL_DART_ID,
        OrbitPolicy, SewError, VolumeIdType,
    },
    geometry::{CoordsFloat, Vertex3},
    stm::{StmClosureResult, Transaction, TransactionClosureResult, abort, try_or_coerce},
};
use nalgebra::Matrix3;
use smallvec::{SmallVec, smallvec};

use crate::utils::compute_tet_orientation;

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
    OpError(#[from] SewError),
    #[error("dart release failed: {0}")]
    DartReleaseError(#[from] DartReleaseError),
    #[error("dart release failed: {0}")]
    DartReservationError(#[from] DartReservationError),
    #[error("cannot extend the cavity: {0}")]
    NonExtendable(&'static str),
}

// -- cavity computation

/// Reduce a cavity until it can be triangulated from its point.
pub fn reduce_to_starshaped_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: Cavity3<T>,
) -> TransactionClosureResult<Cavity3<T>, CavityError> {
    todo!()
}

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
    while let Some((f, [(d1, d1_neigh), (d2, d2_neigh), (d3, d3_neigh)])) =
        boundary.iter().find(|(_, [(d1, _), (d2, _), (d3, _)])| {
            // TODO: use epsilon?
            compute_tet_orientation(t, map, (*d1, *d2, *d3), cavity.point).unwrap() < 0.0
        })
    {
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
                                    if map.beta_tx::<3>(t, *dd).unwrap() == b2 {
                                        Some(*dd_neigh)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap()
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
                                    if map.beta_tx::<3>(t, *dd).unwrap() == b2 {
                                        Some(*dd_neigh)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap()
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
                free_darts.push(d);
            }
        }

        for (f, val) in to_add {
            val.iter().for_each(|(d, d_neigh)| {
                if let Some(val) = boundary.get_mut(&map.face_id_tx(t, *d_neigh).unwrap()) {
                    if let Some(pair) = val.iter_mut().find(|(dd, _)| dd == d_neigh) {
                        pair.1 = *d;
                    } else {
                        panic!()
                    }
                } else {
                    panic!()
                }
            });
            boundary.insert(f, val);
        }
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
                    [fd, map.beta_tx::<1>(t, fd)?, map.beta_tx::<0>(t, fd)?],
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

/// Delete all internal elements of a cavity.
pub fn carve_cavity_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    cavity: Cavity3<T>,
) -> TransactionClosureResult<CarvedCavity3<T>, CavityError> {
    let (cavity_map, cavity_internals) = map_cavity_3d(t, map, &cavity)?;
    let mut free_darts = Vec::new();
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
            free_darts.push(d);
        }
    }

    // necessary for tets that had two or more faces adjacent to the boundary
    for &[(d1, _), (d2, _), (d3, _)] in cavity_map.values() {
        if map.beta_tx::<2>(t, d1)? != NULL_DART_ID {
            try_or_coerce!(map.unsew::<2>(t, d1), CavityError);
        }
        if map.beta_tx::<2>(t, d2)? != NULL_DART_ID {
            try_or_coerce!(map.unsew::<2>(t, d2), CavityError);
        }
        if map.beta_tx::<2>(t, d3)? != NULL_DART_ID {
            try_or_coerce!(map.unsew::<2>(t, d3), CavityError);
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
) -> TransactionClosureResult<(), CavityError> {
    let CarvedCavity3 {
        point,
        boundary,
        mut free_darts,
    } = cavity;
    let n_required_darts = boundary.len() * 9;

    if free_darts.len() < n_required_darts {
        let mut new_darts = try_or_coerce!(
            map.reserve_darts_tx(t, n_required_darts - free_darts.len()),
            CavityError
        );
        free_darts.append(&mut new_darts);
    }
    assert_eq!(free_darts.len() % 9, 0);

    let new_point_dart = free_darts[0];

    for ((_, [(da, da_neigh), (db, db_neigh), (dc, dc_neigh)]), nds) in
        boundary.into_iter().zip(free_darts.chunks_exact(9))
    {
        let nds @ [_, d2, _, _, d5, _, _, d8, _]: [DartIdType; 9] =
            nds.try_into().expect("E: unreachable");
        try_or_coerce!(make_incomplete_tet(t, map, nds), CavityError);

        try_or_coerce!(map.sew::<2>(t, d2, db), CavityError);
        let b2db = map.beta_tx::<2>(t, db_neigh)?;
        if b2db != NULL_DART_ID {
            try_or_coerce!(map.sew::<3>(t, b2db, d2), CavityError);
        }

        try_or_coerce!(map.sew::<2>(t, d5, da), CavityError);
        let b2da = map.beta_tx::<2>(t, da_neigh)?;
        if b2da != NULL_DART_ID {
            try_or_coerce!(map.sew::<3>(t, b2da, d5), CavityError);
        }

        try_or_coerce!(map.sew::<2>(t, d8, dc), CavityError);
        let b2dc = map.beta_tx::<2>(t, dc_neigh)?;
        if b2dc != NULL_DART_ID {
            try_or_coerce!(map.sew::<3>(t, b2dc, d8), CavityError);
        }
    }

    let vid = map.vertex_id_tx(t, new_point_dart)?;
    map.write_vertex(t, vid, point)?;

    Ok(())
}

fn make_incomplete_tet<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    [d1, d2, d3, d4, d5, d6, d7, d8, d9]: [DartIdType; 9],
) -> TransactionClosureResult<(), SewError> {
    // build 3 triangles
    map.sew::<1>(t, d1, d2)?;
    map.sew::<1>(t, d2, d3)?;
    map.sew::<1>(t, d3, d1)?;
    map.sew::<1>(t, d4, d5)?;
    map.sew::<1>(t, d5, d6)?;
    map.sew::<1>(t, d6, d4)?;
    map.sew::<1>(t, d7, d8)?;
    map.sew::<1>(t, d8, d9)?;
    map.sew::<1>(t, d9, d7)?;
    // sew the sides that will be adjacent to the point inserted by the cavity
    map.sew::<2>(t, d3, d4)?;
    map.sew::<2>(t, d6, d7)?;
    map.sew::<2>(t, d9, d1)?;

    Ok(())
}
