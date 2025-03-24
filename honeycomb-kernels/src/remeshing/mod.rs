//! remeshing routine components
//!
//! This module contains all the code used in usual remeshing loops, among which are:
//!
//! - vertex relaxation routines
//! - cell division routines
//! - cell fusion routines
//! - swap-based cell edition routines

use honeycomb_core::{
    attributes::{AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate},
    cmap::{
        CMap2, DartIdType, EdgeIdType, NULL_DART_ID, NULL_EDGE_ID, OrbitPolicy, SewError,
        VertexIdType,
    },
    geometry::{CoordsFloat, Vertex2},
    stm::{StmClosureResult, Transaction, TransactionClosureResult, abort, retry, try_or_coerce},
};
use smallvec::SmallVec;

// -- vertex relaxation

/// Move a vertex to the average of the others' values.
///
/// This function computes the average of a list of coordinates and assigns that value to the
/// specified vertex.
///
/// # Arguments
///
/// - `t: &mut Transaction` -- Associated transaction.
/// - `map: &mut CMap2` -- Edited map.
/// - `vid: VertexIdType` -- Vertex to move.
/// - `others: &[VertexIdType]` -- List of vertex to compute the average from.
///
/// # Errors
///
/// This function will abort and raise an error if the transaction cannot be completed.
///
/// # Panics
///
/// This function may panic if one vertex in the `others` list has no associated coordinates.
///
/// # Example
///
/// For an example of usage, see the `shift` [benchmark code][BENCH].
///
/// [BENCH]: https://github.com/LIHPC-Computational-Geometry/honeycomb/tree/master/benches/src
#[inline]
pub fn move_vertex_to_average<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    vid: VertexIdType,
    others: &[VertexIdType],
) -> StmClosureResult<()> {
    if others.is_empty() {
        return Ok(());
    }
    let mut new_val = Vertex2::default();
    for v in others {
        let vertex = map.read_vertex(t, *v)?.unwrap();
        new_val.0 += vertex.0;
        new_val.1 += vertex.1;
    }
    new_val.0 /= T::from(others.len()).unwrap();
    new_val.1 /= T::from(others.len()).unwrap();
    map.write_vertex(t, vid, new_val)?;
    Ok(())
}

// -- cell insertion

/// Cut an edge in half and build triangles from the new vertex.
///
/// This function takes an edge of the map's boundary as argument, cut it in half, and build two
/// triangles from the new vertex.
///
/// ```text
///
///       +                   +
///      / \                 /|\
///     /   \               / | \
///    /     \     -->     /  |  \
///   /       \           /   |   \
///  /         \         /    |    \
/// +-----------+       +-----+-----+
///       e
///
/// ```
///
/// This function expects to operate on a triangular mesh. At the moment, calling it on another type
/// of mesh may result in non-explicit errors (e.g. an internal sew operation will consistently fail
/// due to a dart being non-free) as there is no check on the face's degree.
///
/// # Arguments
///
/// - `t: &mut Transaction` -- Associated transaction.
/// - `map: &mut CMap2` -- Edited map.
/// - `e: EdgeIdType` -- Edge to cut.
/// - `[nd1, nd2, nd3]: [DartIdType; 3]` -- Free darts used to create the new edges.
///
/// # Errors
///
/// This function will abort and raise an error if:
/// - the transaction cannot be completed,
/// - one of the edge's vertex has no associated coordinates value,
/// - one internal sew operation fails.
///
/// The returned error can be used in conjunction with transaction control to avoid any
/// modifications in case of failure at attribute level. The user can then choose to retry or
/// abort as he wishes using `Transaction::with_control_and_err`.
#[inline]
pub fn cut_outer_edge<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
    [nd1, nd2, nd3]: [DartIdType; 3],
) -> TransactionClosureResult<(), SewError> {
    // unfallible
    try_or_coerce!(map.link::<2>(t, nd1, nd2), SewError);
    try_or_coerce!(map.link::<1>(t, nd2, nd3), SewError);

    let ld = e as DartIdType;
    let (b0ld, b1ld) = (map.beta_transac::<0>(t, ld)?, map.beta_transac::<1>(t, ld)?);

    let (vid1, vid2) = (
        map.vertex_id_transac(t, ld)?,
        map.vertex_id_transac(t, b1ld)?,
    );
    let new_v = match (map.read_vertex(t, vid1)?, map.read_vertex(t, vid2)?) {
        (Some(v1), Some(v2)) => Vertex2::average(&v1, &v2),
        _ => retry()?,
    };
    map.write_vertex(t, nd1, new_v)?;

    map.unsew::<1>(t, ld)?;
    map.unsew::<1>(t, b1ld)?;

    map.sew::<1>(t, ld, nd1)?;
    map.sew::<1>(t, nd1, b0ld)?;
    map.sew::<1>(t, nd3, b1ld)?;
    map.sew::<1>(t, b1ld, nd2)?;

    Ok(())
}

/// Cut an edge in half and build triangles from the new vertex.
///
/// This function takes an edge of the map's as argument, cut it in half, and build four triangles
/// from the new vertex.
///
/// ```text
///
///       +                   +
///      / \                 /|\
///     /   \               / | \
///    /     \             /  |  \
///   /       \           /   |   \
///  /         \         /    |    \
/// +-----------+  -->  +-----+-----+
///  \    e    /         \    |    /
///   \       /           \   |   /
///    \     /             \  |  /
///     \   /               \ | /
///      \ /                 \|/
///       +                   +
///
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
/// - `e: EdgeIdType` -- Edge to cut.
/// - `[nd1, nd2, nd3, nd4, nd5, nd6]: [DartIdType; 6]` -- Free darts used to create the new edges.
///
/// # Errors
///
/// This function will abort and raise an error if:
/// - the transaction cannot be completed,
/// - one of the edge's vertex has no associated coordinates value,
/// - one internal sew operation fails.
///
/// The returned error can be used in conjunction with transaction control to avoid any
/// modifications in case of failure at attribute level. The user can then choose to retry or
/// abort as he wishes using `Transaction::with_control_and_err`.
#[inline]
pub fn cut_inner_edge<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
    [nd1, nd2, nd3, nd4, nd5, nd6]: [DartIdType; 6],
) -> TransactionClosureResult<(), SewError> {
    // unfallible
    try_or_coerce!(map.link::<2>(t, nd1, nd2), SewError);
    try_or_coerce!(map.link::<1>(t, nd2, nd3), SewError);
    try_or_coerce!(map.link::<2>(t, nd4, nd5), SewError);
    try_or_coerce!(map.link::<1>(t, nd5, nd6), SewError);

    let (ld, rd) = (e as DartIdType, map.beta_transac::<2>(t, e as DartIdType)?);
    let (b0ld, b1ld) = (map.beta_transac::<0>(t, ld)?, map.beta_transac::<1>(t, ld)?);
    let (b0rd, b1rd) = (map.beta_transac::<0>(t, rd)?, map.beta_transac::<1>(t, rd)?);

    let (vid1, vid2) = (
        map.vertex_id_transac(t, ld)?,
        map.vertex_id_transac(t, b1ld)?,
    );
    let new_v = match (map.read_vertex(t, vid1)?, map.read_vertex(t, vid2)?) {
        (Some(v1), Some(v2)) => Vertex2::average(&v1, &v2),
        _ => retry()?,
    };
    map.write_vertex(t, nd1, new_v)?;

    map.unsew::<2>(t, ld)?;
    map.unsew::<1>(t, ld)?;
    map.unsew::<1>(t, b1ld)?;
    map.unsew::<1>(t, rd)?;
    map.unsew::<1>(t, b1rd)?;

    map.sew::<2>(t, ld, nd6)?;
    map.sew::<2>(t, rd, nd3)?;

    map.sew::<1>(t, ld, nd1)?;
    map.sew::<1>(t, nd1, b0ld)?;
    map.sew::<1>(t, nd3, b1ld)?;
    map.sew::<1>(t, b1ld, nd2)?;

    map.sew::<1>(t, rd, nd4)?;
    map.sew::<1>(t, nd4, b0rd)?;
    map.sew::<1>(t, nd6, b1rd)?;
    map.sew::<1>(t, b1rd, nd5)?;

    Ok(())
}

// -- cell fusion

/// Error-modeling enum for edge swap routine.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum EdgeCollapseError {
    /// A core operation failed.
    #[error("core operation failed: {0}")]
    FailedCoreOp(SewError),
    /// The edge passed as argument cannot be collapsed due to constraints on its vertices.
    #[error("cannot collapse an edge where both vertices are immovable")]
    NonCollapsableEdge,
    /// The edge passed as argument is null.
    #[error("cannot swap null edge")]
    NullEdge,
    /// One or both of the cells adjacent to the edge are not triangles.
    #[error("cannot swap an edge adjacent to a non-triangular cell")]
    BadTopology,
}

// TODO: use a custom attribute to automatically detect non-collapsable edges
impl From<SewError> for EdgeCollapseError {
    fn from(value: SewError) -> Self {
        EdgeCollapseError::FailedCoreOp(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] // derive ord results in Node < Curve < Surface < Space
enum Anchor {
    Node,
    Curve,
    Surface,
    Space,
}

impl From<Anchor> for OrbitPolicy {
    fn from(value: Anchor) -> Self {
        match value {
            Anchor::Node => OrbitPolicy::Vertex,
            Anchor::Curve => OrbitPolicy::Edge,
            Anchor::Surface => OrbitPolicy::Face,
            Anchor::Space => OrbitPolicy::Volume,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AnchoredVertex2<'a, T: CoordsFloat> {
    d: DartIdType,
    map: &'a CMap2<T>,
    vertex: Vertex2<T>,
    anchor_id: usize,
    anchor: Anchor,
}

impl<T: CoordsFloat> std::fmt::Debug for AnchoredVertex2<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T: CoordsFloat> AttributeBind for AnchoredVertex2<'_, T> {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

impl<T: CoordsFloat> AttributeUpdate for AnchoredVertex2<'_, T> {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        let map = attr1.map;
        let merge_logic = |low: Self, hi: Self| {
            // low is the attribute of the lower-dimensioned anchor
            // hi is the attribute of the higher-dimensioned anchor
            //
            // FIXME: nested STM
            if map.orbit(low.anchor.into(), low.d).any(|d| {
                map.force_read_attribute::<Self>(map.vertex_id(map.beta::<2>(d)))
                    .is_some_and(|v| v.anchor == hi.anchor && v.anchor_id == hi.anchor_id)
            }) {
                Ok(Self { d: low.d })
            } else {
                Err(todo!())
            }
        };
        match attr1.anchor.cmp(&attr2.anchor) {
            std::cmp::Ordering::Equal => {
                if attr1.anchor_id == attr2.anchor_id {
                    Ok(Self {
                        d: attr1.d.min(attr2.d),
                        map: attr1.map,
                        vertex: Vertex2::average(&attr1.vertex, &attr2.vertex),
                        anchor_id: attr1.anchor_id,
                        anchor: attr1.anchor,
                    })
                } else {
                    Err(todo!())
                }
            }
            std::cmp::Ordering::Less => merge_logic(attr1, attr2),
            std::cmp::Ordering::Greater => merge_logic(attr2, attr1),
        }
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        todo!()
    }
}

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
/// # Errors
///
/// This function will abort and raise an error if:
/// - the transaction cannot be completed,
/// - one internal sew operation fails.
///
/// The returned error can be used in conjunction with transaction control to avoid any
/// modifications in case of failure at attribute level. The user can then choose to retry or
/// abort as he wishes using `Transaction::with_control_and_err`.
pub fn collapse_edge<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
) -> TransactionClosureResult<(), EdgeCollapseError> {
    if e == NULL_EDGE_ID {
        abort(EdgeCollapseError::NullEdge)?;
    }
    let (l, r) = (e as DartIdType, map.beta_transac::<2>(t, e as DartIdType)?);

    if r != NULL_DART_ID {
        try_or_coerce!(map.unsew::<2>(t, r), EdgeCollapseError);
        let (b0r, b1r) = (map.beta_transac::<0>(t, r)?, map.beta_transac::<1>(t, r)?);
        if map.beta_transac::<1>(t, b1r)? != b0r {
            abort(EdgeCollapseError::BadTopology)?;
        }

        try_or_coerce!(map.unsew::<1>(t, r), EdgeCollapseError);
        try_or_coerce!(map.unsew::<1>(t, b1r), EdgeCollapseError);
        try_or_coerce!(map.unsew::<1>(t, b0r), EdgeCollapseError);
        let (b2b0r, b2b1r) = (
            map.beta_transac::<2>(t, b0r)?,
            map.beta_transac::<2>(t, b1r)?,
        );
        try_or_coerce!(map.unsew::<2>(t, b0r), EdgeCollapseError);
        try_or_coerce!(map.unsew::<2>(t, b1r), EdgeCollapseError);
        try_or_coerce!(map.sew::<2>(t, b2b0r, b2b1r), EdgeCollapseError);
        // FIXME: set as unused
        // map.remove_free_dart(r);
        // map.remove_free_dart(b0r);
        // map.remove_free_dart(b1r);
    }
    // by this points l is 2-free, whther he was at the beginning or due to the 2-unsew
    let (b0l, b1l) = (map.beta_transac::<0>(t, l)?, map.beta_transac::<1>(t, l)?);
    if map.beta_transac::<1>(t, b1l)? != b0l {
        abort(EdgeCollapseError::BadTopology)?;
    }

    try_or_coerce!(map.unsew::<1>(t, l), EdgeCollapseError);
    try_or_coerce!(map.unsew::<1>(t, b1l), EdgeCollapseError);
    try_or_coerce!(map.unsew::<1>(t, b0l), EdgeCollapseError);
    let (b2b0l, b2b1l) = (
        map.beta_transac::<2>(t, b0l)?,
        map.beta_transac::<2>(t, b1l)?,
    );
    try_or_coerce!(map.unsew::<2>(t, b0l), EdgeCollapseError);
    try_or_coerce!(map.unsew::<2>(t, b1l), EdgeCollapseError);
    try_or_coerce!(map.sew::<2>(t, b2b0l, b2b1l), EdgeCollapseError);
    // FIXME: set as unused
    // map.remove_free_dart(l);
    // map.remove_free_dart(b0l);
    // map.remove_free_dart(b1l);
    Ok(())
}

// -- swap

/// Error-modeling enum for edge swap routine.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum EdgeSwapError {
    /// A core operation failed.
    #[error("core operation failed: {0}")]
    FailedCoreOp(#[from] SewError),
    /// The edge passed as argument is null.
    #[error("cannot swap null edge")]
    NullEdge,
    /// The edge passed as argument is made of single dart, hence doesn't have a cell on each side.
    #[error("cannot swap an edge adjacent to a single cell")]
    IncompleteEdge,
    /// One or both of the cells adjacent to the edge are not triangles.
    #[error("cannot swap an edge adjacent to a non-triangular cell")]
    BadTopology,
}

/// Tip over an edge shared by two triangles.
///
/// The edge is tipped in the clockwise direction. Vertices that were shared become exclusive
/// to each new triangle and vice versa:
///
/// ```text
///
///       +                   +
///      / \                 /|\
///     /   \               / | \
///    /     \             /  |  \
///   /       \           /   |   \
///  /         \         /    |    \
/// +-----------+  -->  +     |     +
///  \    e    /         \    |    /
///   \       /           \   |   /
///    \     /             \  |  /
///     \   /               \ | /
///      \ /                 \|/
///       +                   +
///
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
/// # Panics
///
/// This function will panic if there is no cell on one side of the edge.
///
/// # Errors
///
/// This function will abort and raise an error if:
/// - the transaction cannot be completed,
/// - one internal sew operation fails.
///
/// The returned error can be used in conjunction with transaction control to avoid any
/// modifications in case of failure at attribute level. The user can then choose to retry or
/// abort as he wishes using `Transaction::with_control_and_err`.
#[inline]
pub fn swap_edge<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
) -> TransactionClosureResult<(), EdgeSwapError> {
    if e == NULL_EDGE_ID {
        abort(EdgeSwapError::NullEdge)?;
    }
    let (l, r) = (e as DartIdType, map.beta_transac::<2>(t, e as DartIdType)?);
    if r == NULL_DART_ID {
        abort(EdgeSwapError::IncompleteEdge)?;
    }

    let (b1l, b1r) = (map.beta_transac::<1>(t, l)?, map.beta_transac::<1>(t, r)?);
    let (b0l, b0r) = (map.beta_transac::<0>(t, l)?, map.beta_transac::<0>(t, r)?);
    if map.beta_transac::<1>(t, b1l)? != b0l || map.beta_transac::<1>(t, b1r)? != b0r {
        abort(EdgeSwapError::BadTopology)?;
    }

    try_or_coerce!(map.unsew::<1>(t, l), EdgeSwapError);
    try_or_coerce!(map.unsew::<1>(t, r), EdgeSwapError);
    try_or_coerce!(map.unsew::<1>(t, b0l), EdgeSwapError);
    try_or_coerce!(map.unsew::<1>(t, b0r), EdgeSwapError);
    try_or_coerce!(map.unsew::<1>(t, b1l), EdgeSwapError);
    try_or_coerce!(map.unsew::<1>(t, b1r), EdgeSwapError);

    try_or_coerce!(map.sew::<1>(t, l, b0r), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b0r, b1l), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b1l, l), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, r, b0l), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b0l, b1r), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b1r, r), EdgeSwapError);

    Ok(())
}

#[cfg(test)]
mod tests;
