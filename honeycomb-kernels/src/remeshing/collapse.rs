use honeycomb_core::{
    cmap::{CMap2, DartIdType, EdgeIdType, LinkError, NULL_DART_ID, NULL_EDGE_ID, SewError},
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

impl From<LinkError> for EdgeCollapseError {
    fn from(value: LinkError) -> Self {
        Self::FailedCoreOp(SewError::FailedLink(value))
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
    let (b0l, b1l) = (map.beta_transac::<0>(t, l)?, map.beta_transac::<1>(t, l)?);
    let (b0r, b1r) = (map.beta_transac::<0>(t, r)?, map.beta_transac::<1>(t, r)?);

    if map.beta_transac::<1>(t, b1l)? != b0l {
        abort(EdgeCollapseError::BadTopology)?;
    }
    if r != NULL_DART_ID && map.beta_transac::<1>(t, b1r)? != b0r {
        abort(EdgeCollapseError::BadTopology)?;
    }

    try_or_coerce!(
        collapse_edge_to_midpoint(t, map, (b0l, l, b1l), (b0r, r, b1r)),
        EdgeCollapseError
    );

    Ok(())
}

// -- internals

fn collapse_edge_to_midpoint<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    (b0l, l, b1l): (DartIdType, DartIdType, DartIdType),
    (b0r, r, b1r): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<(), SewError> {
    if r != NULL_DART_ID {
        map.unsew::<2>(t, r)?;
        collapse_halfcell(t, map, (b0r, r, b1r))?;
    }
    // by this point l is 2-free, whether he was at the beginning or due to the 2-unsew
    collapse_halfcell(t, map, (b0l, l, b1l))?;
    Ok(())
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
    // FIXME: set as unused
    // map.remove_free_dart(r);
    // map.remove_free_dart(b0r);
    // map.remove_free_dart(b1r);
    TransactionClosureResult::Ok(())
}

fn collapse_edge_to_base<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    (b0l, l, b1l): (DartIdType, DartIdType, DartIdType), // base == l
    (b0r, r, b1r): (DartIdType, DartIdType, DartIdType),
) -> TransactionClosureResult<(), EdgeCollapseError> {
    let l_vid = map.vertex_id_transac(t, l)?;
    // reading/writing the coordinates to collapse to is easier to handle split/merges correctly
    let tmp = map.read_vertex(t, l_vid)?.expect("no vertex");

    if r != NULL_DART_ID {
        try_or_coerce!(map.unsew::<2>(t, l), EdgeCollapseError);

        let b2b0r = map.beta_transac::<2>(t, b0r)?;
        let b0b2b0r = map.beta_transac::<0>(t, b2b0r)?;
        let b1b2b0r = map.beta_transac::<1>(t, b2b0r)?;

        try_or_coerce!(map.unsew::<1>(t, r), EdgeCollapseError);
        try_or_coerce!(map.unsew::<1>(t, b1r), EdgeCollapseError);
        try_or_coerce!(map.unsew::<1>(t, b0r), EdgeCollapseError);
        if b2b0r != NULL_DART_ID {
            try_or_coerce!(map.unsew::<1>(t, b2b0r), EdgeCollapseError);
            try_or_coerce!(map.unsew::<1>(t, b0b2b0r), EdgeCollapseError);
            try_or_coerce!(map.unlink::<2>(t, b0r), EdgeCollapseError);
            // FIXME: set as unused
            // map.remove_free_dart(r);
            // map.remove_free_dart(b0r);
            // map.remove_free_dart(b2b0r);
            try_or_coerce!(map.sew::<1>(t, b1r, b1b2b0r), EdgeCollapseError);
            try_or_coerce!(map.sew::<1>(t, b0b2b0r, b1r), EdgeCollapseError);
        }
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
        // FIXME: set as unused
        // map.remove_free_dart(l);
        // map.remove_free_dart(b1l);
        // map.remove_free_dart(b2b1l);
        try_or_coerce!(map.sew::<1>(t, b0l, b1b2b1l), EdgeCollapseError);
        try_or_coerce!(map.sew::<1>(t, b0b2b1l, b0l), EdgeCollapseError);
    }

    let new_vid = map.vertex_id_transac(t, b1b2b1l)?;
    map.write_vertex(t, new_vid, tmp)?;

    Ok(())
}
