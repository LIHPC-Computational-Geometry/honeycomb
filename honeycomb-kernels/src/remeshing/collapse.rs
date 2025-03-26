use honeycomb_core::{
    cmap::{CMap2, DartIdType, EdgeIdType, NULL_DART_ID, NULL_EDGE_ID, SewError},
    geometry::CoordsFloat,
    stm::{Transaction, TransactionClosureResult, abort, try_or_coerce},
};

/// Error-modeling enum for edge swap routine.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum EdgeCollapseError {
    /// A core operation failed.
    #[error("core operation failed: {0}")]
    FailedCoreOp(#[from] SewError),
    /// The edge passed as argument cannot be collapsed due to constraints on its vertices.
    #[error("cannot collapse an edge where both vertices are immovable")]
    NonCollapsibleEdge,
    /// The edge passed as argument is null.
    #[error("cannot swap null edge")]
    NullEdge,
    /// One or both of the cells adjacent to the edge are not triangles.
    #[error("cannot swap an edge adjacent to a non-triangular cell")]
    BadTopology,
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
