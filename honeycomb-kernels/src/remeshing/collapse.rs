use honeycomb_core::{
    attributes::{AttributeError, AttributeUpdate},
    cmap::{
        CMap2, DartIdType, EdgeIdType, LinkError, NULL_DART_ID, NULL_EDGE_ID, NULL_VERTEX_ID,
        OrbitPolicy, SewError, VertexIdType,
    },
    geometry::CoordsFloat,
    stm::{Transaction, TransactionClosureResult, abort, retry, try_or_coerce},
};
use smallvec::SmallVec;

use crate::{
    remeshing::{EdgeAnchor, VertexAnchor},
    triangulation::crossp_from_verts,
};

/// Error-modeling enum for edge collapse routine.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum EdgeCollapseError {
    /// A core operation failed.
    #[error("core operation failed: {0}")]
    FailedCoreOp(#[from] SewError),
    /// The edge passed as argument cannot be collapsed due to constraints on its vertices.
    #[error("cannot collapse edge: {0}")]
    NonCollapsibleEdge(&'static str),
    /// The structure after collapse contains a triangle with inverted orientation, making the
    /// geometry invalid.
    #[error("collapsing would result in an inversion of geometry orientation")]
    InvertedOrientation,
    /// The edge passed as argument is null.
    #[error("cannot collapse null edge")]
    NullEdge,
    /// One or both of the cells adjacent to the edge are not triangles.
    #[error("cannot collapse an edge adjacent to a non-triangular cell")]
    BadTopology,
}

impl From<LinkError> for EdgeCollapseError {
    fn from(value: LinkError) -> Self {
        Self::FailedCoreOp(SewError::FailedLink(value))
    }
}

#[allow(clippy::missing_errors_doc)]
/// Collapse an edge separating two triangles.
///
/// ```text
/// +-----+-----+       +-----+-----+
/// |    / \    |        \    |    /
/// |   /   \   |         \  2-3  /
/// 1  2     3  4          1  |  4
/// | /       \ |           \ | /
/// |/         \|            \|/
/// +-----------+  -->        +
/// |\    e    /|            /|\
/// | \       / |           / | \
/// 5  6     7  8          5  |  8
/// |   \   /   |         /  6-7  \
/// |    \ /    |        /    |    \
/// +-----+-----+       +-----+-----+
/// ```
///
/// This function expects to operate on a triangular mesh. The edge may be collapsed to one of
/// the existing vertices, or to the average of their value; this is determined by the anchoring
/// of the mesh to its geometry. If no anchoring attributes are present, the edge is always
/// collapsed to the average value.
///
/// # Arguments
///
/// - `t: &mut Transaction` -- Associated transaction.
/// - `map: &mut CMap2` -- Edited map.
/// - `e: EdgeIdType` -- Edge to move.
///
/// # Return / Errors
///
/// Upon success, this function will return the ID of the new vertex formed after the collapse.
/// Depending on the anchoring constraints, it may be placed on one of the two previously existing
/// vertex, or to their average value.
///
/// This function will abort and raise an error if:
/// - the transaction cannot be completed,
/// - one internal sew operation fails,
/// - the collapse cannot be completed; see [`EdgeCollapseError`] for more information.
///
/// The returned error can be used in conjunction with transaction control to avoid any
/// modifications in case of failure at attribute level. The user can then choose to retry or
/// abort as he wishes using `Transaction::with_control_and_err`.
///
/// <div class="warning">
/// Note that the function will return `StmError::Retry` if it attempts to read a missing vertex.
/// If used within a transaction which retries indefinitely (e.g. `atomically_with_err`), it can
/// lead to an infinite loop.
///
/// This will not happen unless the map ends up in an incorrect state where topological vertices
/// have no associated coordinates.
/// </div>
#[allow(clippy::many_single_char_names)]
pub fn collapse_edge<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
) -> TransactionClosureResult<VertexIdType, EdgeCollapseError> {
    if e == NULL_EDGE_ID {
        abort(EdgeCollapseError::NullEdge)?;
    }
    let (l, r) = (e as DartIdType, map.beta_transac::<2>(t, e as DartIdType)?);
    let (b0l, b1l) = (map.beta_transac::<0>(t, l)?, map.beta_transac::<1>(t, l)?);
    let (b0r, b1r) = (map.beta_transac::<0>(t, r)?, map.beta_transac::<1>(t, r)?);

    if map.beta_transac::<1>(t, b1l)? != b0l {
        abort(EdgeCollapseError::BadTopology)?;
    }
    if r != NULL_DART_ID && map.beta_transac::<1>(t, b1r)? != b0r {
        abort(EdgeCollapseError::BadTopology)?;
    }

    let new_vid = match is_collapsible(t, map, e)? {
        Collapsible::Average => try_or_coerce!(
            collapse_edge_to_midpoint(t, map, (b0l, l, b1l), (b0r, r, b1r)),
            EdgeCollapseError
        ),
        Collapsible::Left => try_or_coerce!(
            collapse_edge_to_base(t, map, (b0l, l, b1l), (b0r, r, b1r)),
            EdgeCollapseError
        ),
        Collapsible::Right => try_or_coerce!(
            collapse_edge_to_base(t, map, (b0r, r, b1r), (b0l, l, b1l)),
            EdgeCollapseError
        ),
    };

    check_orbit_orientation(t, map, new_vid)?;

    Ok(new_vid)
}

// -- internals

// ---- collapse criteria

#[derive(Debug)]
enum Collapsible {
    Average,
    Left,
    Right,
}

fn is_collapsible<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
) -> TransactionClosureResult<Collapsible, EdgeCollapseError> {
    if !map.contains_attribute::<VertexAnchor>() {
        // if there are no anchors, we'll assume we can naively collapse
        return Ok(Collapsible::Average);
    }
    let (l, b1l) = (e as DartIdType, map.beta_transac::<1>(t, e as DartIdType)?);

    // first check anchor predicates

    let (l_vid, r_vid) = (map.vertex_id_transac(t, l)?, map.vertex_id_transac(t, b1l)?);
    let (l_anchor, r_anchor, edge_anchor) = match (
        map.read_attribute::<VertexAnchor>(t, l_vid)?,
        map.read_attribute::<VertexAnchor>(t, r_vid)?,
        map.read_attribute::<EdgeAnchor>(t, e)?,
    ) {
        (Some(a1), Some(a2), Some(a3)) => (a1, a2, a3),
        _ => retry()?,
    };

    match AttributeUpdate::merge(l_anchor, r_anchor) {
        Ok(val) => {
            // check ID too? did other checks filter that out already?
            // does having different IDs here mean the classification is bad?
            if edge_anchor.anchor_dim() == l_anchor.anchor_dim()
                || edge_anchor.anchor_dim() == r_anchor.anchor_dim()
            {
                match (val == l_anchor, val == r_anchor) {
                    (true, true) => Ok(Collapsible::Average),
                    (true, false) => Ok(Collapsible::Left),
                    (false, true) => Ok(Collapsible::Right),
                    (false, false) => unreachable!(),
                }
            } else {
                abort(EdgeCollapseError::NonCollapsibleEdge(
                    "collapsing along this edge is impossible",
                ))
            }
        }
        Err(AttributeError::FailedMerge(_, _)) => abort(EdgeCollapseError::NonCollapsibleEdge(
            "vertex have incompatible anchors",
        )),
        Err(AttributeError::FailedSplit(_, _) | AttributeError::InsufficientData(_, _)) => {
            unreachable!();
        }
    }
}

fn check_orbit_orientation<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    vid: VertexIdType,
) -> TransactionClosureResult<(), EdgeCollapseError> {
    if let Some(new_v) = map.read_vertex(t, vid)? {
        let mut tmp: SmallVec<DartIdType, 10> = SmallVec::new();
        for d in map.orbit_transac(t, OrbitPolicy::Vertex, vid) {
            tmp.push(d?);
        }

        let ref_sign = {
            let d = tmp[0];
            let b1d = map.beta_transac::<1>(t, d)?;
            let b1b1d = map.beta_transac::<1>(t, b1d)?;
            let vid1 = map.vertex_id_transac(t, b1d)?;
            let vid2 = map.vertex_id_transac(t, b1b1d)?;
            let v1 = if let Some(v) = map.read_vertex(t, vid1)? {
                v
            } else {
                retry()?
            };
            let v2 = if let Some(v) = map.read_vertex(t, vid2)? {
                v
            } else {
                retry()?
            };

            let crossp = crossp_from_verts(&new_v, &v1, &v2);
            crossp.signum()
        };
        for &d in &tmp[1..] {
            let b1d = map.beta_transac::<1>(t, d)?;
            let b1b1d = map.beta_transac::<1>(t, b1d)?;
            let vid1 = map.vertex_id_transac(t, b1d)?;
            let vid2 = map.vertex_id_transac(t, b1b1d)?;
            let v1 = if let Some(v) = map.read_vertex(t, vid1)? {
                v
            } else {
                retry()?
            };
            let v2 = if let Some(v) = map.read_vertex(t, vid2)? {
                v
            } else {
                retry()?
            };

            let crossp = crossp_from_verts(&new_v, &v1, &v2);

            if ref_sign != crossp.signum() {
                abort(EdgeCollapseError::InvertedOrientation)?;
            }
        }
    } else {
        retry()?;
    }

    Ok(())
}
// ---- midpoint collapse

fn collapse_edge_to_midpoint<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    (b0l, l, b1l): (DartIdType, DartIdType, DartIdType),
    (b0r, r, b1r): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<VertexIdType, SewError> {
    if r != NULL_DART_ID {
        map.unsew::<2>(t, r)?;
        collapse_halfcell_to_midpoint(t, map, (b0r, r, b1r))?;
    }
    // by this point l is 2-free, whether he was at the beginning or due to the 2-unsew
    let b2b0l = map.beta_transac::<2>(t, b0l)?; // save this before left cell collapse
    collapse_halfcell_to_midpoint(t, map, (b0l, l, b1l))?;

    Ok(if b2b0l != NULL_DART_ID {
        map.vertex_id_transac(t, b2b0l)?
    } else if r != NULL_DART_ID {
        map.vertex_id_transac(t, b1r)?
    } else {
        // this can happen from a valid configuration, so we handle it
        NULL_VERTEX_ID
    })
}

fn collapse_halfcell_to_midpoint<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    (b0d, d, b1d): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<(), SewError> {
    map.unsew::<1>(t, d)?;
    map.unsew::<1>(t, b1d)?;
    map.unsew::<1>(t, b0d)?;
    let (b2b0d, b2b1d) = (
        map.beta_transac::<2>(t, b0d)?,
        map.beta_transac::<2>(t, b1d)?,
    );
    match (b2b0d == NULL_DART_ID, b2b1d == NULL_DART_ID) {
        (false, false) => {
            map.unsew::<2>(t, b0d)?;
            map.unsew::<2>(t, b1d)?;
            map.sew::<2>(t, b2b0d, b2b1d)?;
        }
        (true, false) => {
            map.unsew::<2>(t, b1d)?;
        }
        (false, true) => {
            map.unsew::<2>(t, b0d)?;
        }
        (true, true) => {}
    }

    map.remove_free_dart_transac(t, d)?;
    map.remove_free_dart_transac(t, b0d)?;
    map.remove_free_dart_transac(t, b1d)?;
    TransactionClosureResult::Ok(())
}

// ---- base collapse

fn collapse_edge_to_base<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    (b0l, l, b1l): (DartIdType, DartIdType, DartIdType), // base == l
    (b0r, r, b1r): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<VertexIdType, EdgeCollapseError> {
    // reading/writing the coordinates to collapse to is easier to handle split/merges correctly
    let l_vid = map.vertex_id_transac(t, l)?;
    let tmp_vertex = map.read_vertex(t, l_vid)?;
    let tmp_anchor = map.read_attribute::<VertexAnchor>(t, l_vid)?;

    if r != NULL_DART_ID {
        try_or_coerce!(map.unsew::<2>(t, l), EdgeCollapseError);
        try_or_coerce!(
            collapse_halfcell_to_base(t, map, (b1r, r, b0r)),
            EdgeCollapseError
        );
    }
    // by this point l is 2-free, whether he was at the beginning or due to the 2-unsew
    let b2b0l = map.beta_transac::<2>(t, b0l)?; // save this before left cell collapse
    try_or_coerce!(
        collapse_halfcell_to_base(t, map, (b0l, l, b1l)),
        EdgeCollapseError
    );

    let new_vid = if b2b0l != NULL_DART_ID {
        map.vertex_id_transac(t, b2b0l)?
    } else if r != NULL_DART_ID {
        map.vertex_id_transac(t, b1r)?
    } else {
        // this can happen from a valid configuration, so we handle it
        NULL_VERTEX_ID
    };

    if new_vid != NULL_VERTEX_ID {
        if let Some(v) = tmp_vertex {
            map.write_vertex(t, new_vid, v)?;
        } // else eprintln! ?
        if let Some(a) = tmp_anchor {
            map.write_attribute(t, new_vid, a)?;
        }
    }

    Ok(new_vid)
}

fn collapse_halfcell_to_base<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    // d_previous_edge, d_edge, d_next_edge
    (d_pe, d_e, d_ne): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<(), SewError> {
    let b2d_ne = map.beta_transac::<2>(t, d_ne)?;
    let b0b2d_ne = map.beta_transac::<0>(t, b2d_ne)?;
    let b1b2d_ne = map.beta_transac::<1>(t, b2d_ne)?;

    map.unsew::<1>(t, d_e)?;
    map.unsew::<1>(t, d_pe)?;
    map.unsew::<1>(t, d_ne)?;
    if b2d_ne == NULL_DART_ID {
        map.unsew::<2>(t, d_pe)?;
        map.remove_free_dart_transac(t, d_pe)?;
    } else {
        map.unsew::<1>(t, b2d_ne)?;
        map.unsew::<1>(t, b0b2d_ne)?;
        try_or_coerce!(map.unlink::<2>(t, d_ne), SewError);
        map.remove_free_dart_transac(t, b2d_ne)?;
        map.sew::<1>(t, d_pe, b1b2d_ne)?;
        map.sew::<1>(t, b0b2d_ne, d_pe)?;
    }
    map.remove_free_dart_transac(t, d_e)?;
    map.remove_free_dart_transac(t, d_ne)?;

    Ok(())
}
