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

/// Error-modeling enum for edge swap routine.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum EdgeCollapseError {
    /// A core operation failed.
    #[error("core operation failed: {0}")]
    FailedCoreOp(#[from] SewError),
    /// The edge passed as argument cannot be collapsed due to constraints on its vertices.
    #[error("cannot collapse edge: {0}")]
    NonCollapsibleEdge(&'static str),
    /// The edge passed as argument is null.
    #[error("cannot swap null edge")]
    NullEdge,
    /// One or both of the cells adjacent to the edge are not triangles.
    #[error("cannot swap an edge adjacent to a non-triangular cell")]
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
/// This function expects to operate on a triangular mesh. At the moment, calling it on another type
/// of mesh may result in non-explicit errors (e.g. an internal sew operation will consistently fail
/// due to a dart being non-free) as there is no check on each faces' degree.
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

    let new_v = if let Some(v) = map.read_vertex(t, new_vid)? {
        v
    } else {
        retry()?
    };
    let mut tmp: SmallVec<DartIdType, 10> = SmallVec::new();
    for d in map.orbit_transac(t, OrbitPolicy::Vertex, new_vid) {
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
            abort(EdgeCollapseError::NonCollapsibleEdge(
                "resulting geometry is inverted",
            ))?;
        }
    }

    Ok(new_vid)
}

// -- internals

fn collapse_edge_to_midpoint<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    (b0l, l, b1l): (DartIdType, DartIdType, DartIdType),
    (b0r, r, b1r): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<VertexIdType, SewError> {
    let mut tmp_d = NULL_DART_ID;
    if r != NULL_DART_ID {
        tmp_d = b1r;
        map.unsew::<2>(t, r)?;
        collapse_halfcell(t, map, (b0r, r, b1r))?;
    }
    // by this point l is 2-free, whether he was at the beginning or due to the 2-unsew
    let b2b0l = map.beta_transac::<2>(t, b0l)?;
    if b2b0l != NULL_DART_ID {
        tmp_d = b2b0l;
    }
    collapse_halfcell(t, map, (b0l, l, b1l))?;

    Ok(if tmp_d == NULL_DART_ID {
        NULL_VERTEX_ID
    } else {
        map.vertex_id_transac(t, tmp_d)?
    })
}

fn collapse_halfcell<T: CoordsFloat>(
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
    map.unsew::<2>(t, b0d)?;
    map.unsew::<2>(t, b1d)?;
    map.sew::<2>(t, b2b0d, b2b1d)?;
    map.remove_free_dart_transac(t, d)?;
    map.remove_free_dart_transac(t, b0d)?;
    map.remove_free_dart_transac(t, b1d)?;
    TransactionClosureResult::Ok(())
}

fn collapse_edge_to_base<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    (b0l, l, b1l): (DartIdType, DartIdType, DartIdType), // base == l
    (b0r, r, b1r): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<VertexIdType, EdgeCollapseError> {
    let l_vid = map.vertex_id_transac(t, l)?;
    // reading/writing the coordinates to collapse to is easier to handle split/merges correctly
    let tmp_vertex = if let Some(v) = map.read_vertex(t, l_vid)? {
        v
    } else {
        retry()?
    };
    // remove condition? do we expect to use this without anchors?
    let tmp_anchor = if map.contains_attribute::<VertexAnchor>() {
        map.read_attribute::<VertexAnchor>(t, l_vid)?
    } else {
        None
    };
    let mut tmp_d = NULL_DART_ID;

    if r != NULL_DART_ID {
        try_or_coerce!(map.unsew::<2>(t, l), EdgeCollapseError);

        let b2b0r = map.beta_transac::<2>(t, b0r)?;
        let b0b2b0r = map.beta_transac::<0>(t, b2b0r)?;
        let b1b2b0r = map.beta_transac::<1>(t, b2b0r)?;

        try_or_coerce!(map.unsew::<1>(t, r), EdgeCollapseError);
        try_or_coerce!(map.unsew::<1>(t, b1r), EdgeCollapseError);
        try_or_coerce!(map.unsew::<1>(t, b0r), EdgeCollapseError);
        tmp_d = b1r;
        if b2b0r != NULL_DART_ID {
            try_or_coerce!(map.unsew::<1>(t, b2b0r), EdgeCollapseError);
            try_or_coerce!(map.unsew::<1>(t, b0b2b0r), EdgeCollapseError);
            try_or_coerce!(map.unlink::<2>(t, b0r), EdgeCollapseError);
            map.remove_free_dart_transac(t, r)?;
            map.remove_free_dart_transac(t, b0r)?;
            map.remove_free_dart_transac(t, b2b0r)?;
            try_or_coerce!(map.sew::<1>(t, b1r, b1b2b0r), EdgeCollapseError);
            try_or_coerce!(map.sew::<1>(t, b0b2b0r, b1r), EdgeCollapseError);
        }
    }

    let b2b0l = map.beta_transac::<2>(t, b0l)?;
    if b2b0l != NULL_DART_ID {
        tmp_d = b2b0l;
    }
    let b2b1l = map.beta_transac::<2>(t, b1l)?;
    let b0b2b1l = map.beta_transac::<0>(t, b2b1l)?;
    let b1b2b1l = map.beta_transac::<1>(t, b2b1l)?;

    try_or_coerce!(map.unsew::<1>(t, l), EdgeCollapseError);
    try_or_coerce!(map.unsew::<1>(t, b0l), EdgeCollapseError);
    try_or_coerce!(map.unsew::<1>(t, b1l), EdgeCollapseError);
    if b2b1l != NULL_DART_ID {
        try_or_coerce!(map.unsew::<1>(t, b2b1l), EdgeCollapseError);
        try_or_coerce!(map.unsew::<1>(t, b0b2b1l), EdgeCollapseError);
        try_or_coerce!(map.unlink::<2>(t, b1l), EdgeCollapseError);
        map.remove_free_dart_transac(t, l)?;
        map.remove_free_dart_transac(t, b1l)?;
        map.remove_free_dart_transac(t, b2b1l)?;
        try_or_coerce!(map.sew::<1>(t, b0l, b1b2b1l), EdgeCollapseError);
        try_or_coerce!(map.sew::<1>(t, b0b2b1l, b0l), EdgeCollapseError);
    }

    let new_vid = map.vertex_id_transac(t, tmp_d)?;
    map.write_vertex(t, new_vid, tmp_vertex)?;
    if let Some(a) = tmp_anchor {
        map.write_attribute(t, new_vid, a)?;
    }

    Ok(new_vid)
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
    let (l_anchor, r_anchor, edge_anchor) = if let (Some(a1), Some(a2), Some(a3)) = (
        map.read_attribute::<VertexAnchor>(t, l_vid)?,
        map.read_attribute::<VertexAnchor>(t, r_vid)?,
        map.read_attribute::<EdgeAnchor>(t, e)?,
    ) {
        (a1, a2, a3)
    } else {
        retry()?
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
                    "edge's anchor prevents collapsing these vertices",
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

enum Collapsible {
    Average,
    Left,
    Right,
}
