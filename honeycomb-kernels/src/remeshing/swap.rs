use honeycomb_core::{
    cmap::{CMap2, DartIdType, EdgeIdType, NULL_DART_ID, NULL_EDGE_ID, SewError},
    geometry::CoordsFloat,
    stm::{Transaction, TransactionClosureResult, abort, try_or_coerce},
};

use crate::utils::{FaceAnchor, VertexAnchor};

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
    let l_fid = map.face_id_transac(t, l)?;
    let r_fid = map.face_id_transac(t, r)?;
    let (l_a, r_a) = if map.contains_attribute::<FaceAnchor>() {
        let l_a = map.remove_attribute::<FaceAnchor>(t, l_fid)?;
        let r_a = map.remove_attribute::<FaceAnchor>(t, r_fid)?;
        if l_a != r_a {
            abort(todo!("add non-swapable error variant"))?;
        }
        (l_a, r_a)
    } else {
        (None, None)
    };

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

    if map.contains_attribute::<VertexAnchor>() {
        let l_vid = map.vertex_id_transac(t, l)?;
        let r_vid = map.vertex_id_transac(t, r)?;
        map.remove_attribute::<VertexAnchor>(t, l_vid)?;
        map.remove_attribute::<VertexAnchor>(t, r_vid)?;
    }

    try_or_coerce!(map.sew::<1>(t, l, b0r), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b0r, b1l), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b1l, l), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, r, b0l), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b0l, b1r), EdgeSwapError);
    try_or_coerce!(map.sew::<1>(t, b1r, r), EdgeSwapError);

    // update face ids after topology change
    let l_fid = map.face_id_transac(t, l)?;
    let r_fid = map.face_id_transac(t, r)?;
    // rewrite face attributes
    match (l_a, r_a) {
        (Some(l_a), Some(r_a)) => {
            map.write_attribute(t, l_fid, l_a)?;
            map.write_attribute(t, r_fid, r_a)?;
        }
        (Some(_), None) | (None, Some(_)) => unreachable!(),
        (None, None) => {}
    }

    Ok(())
}
