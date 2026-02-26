//! Vertex isnertion routines

use honeycomb_core::{
    attributes::AttributeError,
    cmap::{CMap2, DartIdType, EdgeIdType, LinkError, NULL_DART_ID, SewError},
    geometry::{CoordsFloat, Vertex2},
    stm::{Transaction, TransactionClosureResult, abort, try_or_coerce},
};

// -- error type

/// Error-modeling enum for vertex insertion routines.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum VertexInsertionError {
    /// A core operation failed.
    #[error("core operation failed: {0}")]
    FailedCoreOp(#[from] SewError),
    /// Relative position of the new vertex isn't located on the edge.
    #[error("vertex placement for split is not in ]0;1[")]
    VertexBound,
    /// One or both vertices of the edge are undefined.
    #[error("edge isn't defined correctly")]
    UndefinedEdge,
    /// Darts passed to the function do not match requirements.
    #[error("passed darts should be free & non-null - {0}")]
    InvalidDarts(&'static str),
    /// The number of darts passed to create the new segments is too low. The `usize` value
    /// is the number of missing darts.
    #[error("wrong # of darts - expected `{0}`, got {1}")]
    WrongAmountDarts(usize, usize),
}

impl From<LinkError> for VertexInsertionError {
    fn from(value: LinkError) -> Self {
        Self::FailedCoreOp(value.into())
    }
}

impl From<AttributeError> for VertexInsertionError {
    fn from(value: AttributeError) -> Self {
        Self::FailedCoreOp(value.into())
    }
}

// -- routines

/// Insert a vertex in an edge, cutting it into two segments.
///
/// <div class="warning">
/// This implementation is 2D specific.
/// </div>
///
/// # Arguments
///
/// - `cmap: &mut CMap2<T>` -- Reference to the modified map.
/// - `t: &mut Transaction` -- Associated transaction.
/// - `edge_id: EdgeIdentifier` -- Target edge.
/// - `new_darts: (DartIdentifier, DartIdentifier)` -- Dart IDs used to build the new vertex/segments.
/// - `midpoint_vertex: Option<T>` -- Relative position of the new vertex, starting from the
///   vertex of the dart sharing `edge_id` as its identifier.
///
/// ## Dart IDs Requirements & Usage
///
/// Because of the dimension, the number of dart needed to perform this operation is at most
/// two. These are the requirements for these two darts:
/// - identifiers are passed as a tuple.
/// - the first dart of the tuple will always be used if the operation is successful.
/// - the second dart of the tuple will only be used if the original edge is made of two darts;
///   if that is not the case, the second dart ID can be `NULL_DART_ID`.
/// - both of these darts should be free
///
/// # Errors
///
/// This function will abort and raise an error if:
/// - the transaction cannot be completed,
/// - a hypothesis over input isn't verified (see [`VertexInsertionError`]),
/// - an internal operation failed.
///
/// The returned error can be used in conjunction with transaction control to avoid any
/// modifications in case of failure at attribute level. The user can then choose to retry or
/// abort as he wishes using `Transaction::with_control_and_err`.
#[allow(clippy::too_many_lines)]
pub fn insert_vertex_on_edge<T: CoordsFloat>(
    cmap: &CMap2<T>,
    t: &mut Transaction,
    edge_id: EdgeIdType,
    new_darts: (DartIdType, DartIdType), // 2D => statically known number of darts
    midpoint_vertex: Option<T>,
) -> TransactionClosureResult<(), VertexInsertionError> {
    // midpoint check
    if midpoint_vertex.is_some_and(|p| (p >= T::one()) | (p <= T::zero())) {
        abort(VertexInsertionError::VertexBound)?;
    }

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdType;
    let base_dart2 = cmap.beta_tx::<2>(t, base_dart1)?;

    if new_darts.0 == NULL_DART_ID || !cmap.is_free_tx(t, new_darts.0)? {
        abort(VertexInsertionError::InvalidDarts(
            "first dart is null or not free",
        ))?;
    }
    if base_dart2 != NULL_DART_ID
        && (new_darts.1 == NULL_DART_ID || !cmap.is_free_tx(t, new_darts.1)?)
    {
        abort(VertexInsertionError::InvalidDarts(
            "second dart is null or not free",
        ))?;
    }

    // base darts making up the edge
    let base_dart2 = cmap.beta_tx::<2>(t, base_dart1)?;
    if base_dart2 == NULL_DART_ID {
        let b1d1_old = cmap.beta_tx::<1>(t, base_dart1)?;
        let b1d1_new = new_darts.0;
        let (vid1, vid2) = (
            cmap.vertex_id_tx(t, base_dart1)?,
            cmap.vertex_id_tx(t, b1d1_old)?,
        );
        let (Some(v1), Some(v2)) = (cmap.read_vertex_tx(t, vid1)?, cmap.read_vertex_tx(t, vid2)?)
        else {
            abort(VertexInsertionError::UndefinedEdge)?
        };
        // unsew current dart
        if b1d1_old != NULL_DART_ID {
            try_or_coerce!(cmap.unlink_tx::<1>(t, base_dart1), VertexInsertionError);
        }
        // rebuild the edge
        try_or_coerce!(
            cmap.link_tx::<1>(t, base_dart1, b1d1_new),
            VertexInsertionError
        );
        try_or_coerce!(
            cmap.link_tx::<1>(t, b1d1_new, b1d1_old),
            VertexInsertionError
        );
        // insert the new vertex
        let seg = v2 - v1;
        let vnew = cmap.vertex_id_tx(t, b1d1_new)?;
        cmap.write_vertex_tx(
            t,
            vnew,
            midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
        )?;
        Ok(())
    } else {
        let b1d1_old = cmap.beta_tx::<1>(t, base_dart1)?;
        let b1d2_old = cmap.beta_tx::<1>(t, base_dart2)?;
        let (b1d1_new, b1d2_new) = new_darts;
        let (vid1, vid2) = (
            cmap.vertex_id_tx(t, base_dart1)?,
            cmap.vertex_id_tx(t, base_dart2)?,
        );
        let (Some(v1), Some(v2)) = (cmap.read_vertex_tx(t, vid1)?, cmap.read_vertex_tx(t, vid2)?)
        else {
            abort(VertexInsertionError::UndefinedEdge)?
        };
        // unsew current darts
        if b1d1_old != NULL_DART_ID {
            try_or_coerce!(cmap.unlink_tx::<1>(t, base_dart1), VertexInsertionError);
        }
        if b1d2_old != NULL_DART_ID {
            try_or_coerce!(cmap.unlink_tx::<1>(t, base_dart2), VertexInsertionError);
        }
        // cmap.set_beta::<1>(base_dart1, 0);
        // cmap.set_beta::<0>(b1d1_old, 0);
        // cmap.set_beta::<1>(base_dart2, 0);
        // cmap.set_beta::<0>(b1d2_old, 0);
        try_or_coerce!(cmap.unlink_tx::<2>(t, base_dart1), VertexInsertionError);
        // rebuild the edge
        try_or_coerce!(
            cmap.link_tx::<1>(t, base_dart1, b1d1_new),
            VertexInsertionError
        );
        if b1d1_old != NULL_DART_ID {
            try_or_coerce!(
                cmap.link_tx::<1>(t, b1d1_new, b1d1_old),
                VertexInsertionError
            );
        }
        try_or_coerce!(
            cmap.link_tx::<1>(t, base_dart2, b1d2_new),
            VertexInsertionError
        );
        if b1d2_old != NULL_DART_ID {
            try_or_coerce!(
                cmap.link_tx::<1>(t, b1d2_new, b1d2_old),
                VertexInsertionError
            );
        }
        try_or_coerce!(
            cmap.link_tx::<2>(t, base_dart1, b1d2_new),
            VertexInsertionError
        );
        try_or_coerce!(
            cmap.link_tx::<2>(t, base_dart2, b1d1_new),
            VertexInsertionError
        );
        // insert the new vertex
        let seg = v2 - v1;
        let vnew = cmap.vertex_id_tx(t, b1d1_new)?;
        cmap.write_vertex_tx(
            t,
            vnew,
            midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
        )?;
        Ok(())
    }
}

#[allow(clippy::missing_errors_doc)]
/// Insert `n` vertices in an edge, cutting it into `n+1` segments.
///
/// <div class="warning">
/// This implementation is 2D specific.
/// </div>
///
/// # Arguments
///
/// - `cmap: &mut CMap2<T>` -- Reference to the modified map.
/// - `t: &mut Transaction` -- Associated transaction.
/// - `edge_id: EdgeIdentifier` -- Target edge.
/// - `new_darts: &[DartIdentifier]` -- Dart IDs used to build the new vertices/segments.
/// - `midpoint_vertices: &[T]` -- Relative positions of new vertices, starting from the
///   vertex of the dart sharing `edge_id` as its identifier.
///
/// ## Dart IDs Requirements & Usage
///
/// Because of the dimension, we can easily compute the number of dart needed to perform this
/// operation. These are the requirements for the darts:
/// - identifiers are passed as a slice:
///   - slice length should verify `new_darts.len() == 2 * midpoint_vertices.len()`
/// - the first half of the slice will always be used if the operation is successful.
/// - the second half of the slice will only be used if the original edge is made of two darts;
///   if that is not the case, the second half IDs can all be `NULL_DART_ID`s.
/// - all of these darts should be free
///
/// # Errors
///
/// This function will abort and raise an error if:
/// - the transaction cannot be completed,
/// - a hypothesis over input isn't verified (see [`VertexInsertionError`]),
/// - an internal operation failed.
///
/// The returned error can be used in conjunction with transaction control to avoid any
/// modifications in case of failure at attribute level. The user can then choose to retry or
/// abort as he wishes using `Transaction::with_control_and_err`.
///
/// # Example
///
/// ```
/// # use honeycomb_core::cmap::{CMap2, CMapBuilder, NULL_DART_ID};
/// # use honeycomb_core::geometry::Vertex2;
/// # use honeycomb_core::stm::atomically_with_err;
/// # use honeycomb_kernels::cell_insertion::insert_vertices_on_edge;
/// // before
/// //    <--2---
/// //  1         2
/// //    ---1-->
///
/// let mut map: CMap2<_> = CMapBuilder::<2>::from_n_darts(2)
///                             .build()
///                             .unwrap();
/// map.force_link::<2>(1, 2);
/// map.force_write_vertex(1, (0.0, 0.0));
/// map.force_write_vertex(2, (1.0, 0.0));
///
/// let nd = map.allocate_unused_darts(6);
///
/// // split
/// assert!(
///     atomically_with_err(|t| insert_vertices_on_edge(
///         &map,
///         t,
///         1,
///         &[nd, nd + 1, nd + 2, nd + 3, nd + 4, nd + 5],
///         &[0.25, 0.50, 0.75],
///     )).is_ok()
/// );
///
/// // after
/// //    <-<-<-<
/// //  1 -3-4-5- 2
/// //    >->->->
///
/// // checks
/// let new_darts = [
///     map.beta::<1>(1),
///     map.beta::<1>(map.beta::<1>(1)),
///     map.beta::<1>(map.beta::<1>(map.beta::<1>(1))),
/// ];
/// assert_eq!(&new_darts, &[3, 4, 5]);
/// assert_eq!(map.force_read_vertex(3), Some(Vertex2(0.25, 0.0)));
/// assert_eq!(map.force_read_vertex(4), Some(Vertex2(0.50, 0.0)));
/// assert_eq!(map.force_read_vertex(5), Some(Vertex2(0.75, 0.0)));
///
/// assert_eq!(map.beta::<1>(1), 3);
/// assert_eq!(map.beta::<1>(3), 4);
/// assert_eq!(map.beta::<1>(4), 5);
/// assert_eq!(map.beta::<1>(5), NULL_DART_ID);
///
/// assert_eq!(map.beta::<1>(2), 6);
/// assert_eq!(map.beta::<1>(6), 7);
/// assert_eq!(map.beta::<1>(7), 8);
/// assert_eq!(map.beta::<1>(8), NULL_DART_ID);
///
/// assert_eq!(map.beta::<2>(1), 8);
/// assert_eq!(map.beta::<2>(3), 7);
/// assert_eq!(map.beta::<2>(4), 6);
/// assert_eq!(map.beta::<2>(5), 2);
/// ```
pub fn insert_vertices_on_edge<T: CoordsFloat>(
    cmap: &CMap2<T>,
    t: &mut Transaction,
    edge_id: EdgeIdType,
    new_darts: &[DartIdType],
    midpoint_vertices: &[T],
) -> TransactionClosureResult<(), VertexInsertionError> {
    // check pre-allocated darts reqs
    let n_t = midpoint_vertices.len();
    let n_d = new_darts.len();
    if n_d != 2 * n_t {
        abort(VertexInsertionError::WrongAmountDarts(2 * n_t, n_d))?;
    }
    for d in new_darts {
        if !cmap.is_free_tx(t, *d)? {
            abort(VertexInsertionError::InvalidDarts("one dart is not free"))?;
        }
    }
    // get the first and second halves
    let darts_fh = &new_darts[..n_t];
    let darts_sh = &new_darts[n_t..];

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdType;
    let base_dart2 = cmap.beta_tx::<2>(t, base_dart1)?;

    if darts_fh.contains(&NULL_DART_ID) {
        abort(VertexInsertionError::InvalidDarts(
            "one dart of the first half is null",
        ))?;
    }
    if base_dart2 != NULL_DART_ID && darts_sh.contains(&NULL_DART_ID) {
        abort(VertexInsertionError::InvalidDarts(
            "one dart of the second half is null",
        ))?;
    }

    if midpoint_vertices
        .iter()
        .any(|p| (*p >= T::one()) | (*p <= T::zero()))
    {
        abort(VertexInsertionError::VertexBound)?;
    }

    let base_dart2 = cmap.beta_tx::<2>(t, base_dart1)?;
    let b1d1_old = cmap.beta_tx::<1>(t, base_dart1)?;

    let (vid1, vid2) = (
        cmap.vertex_id_tx(t, base_dart1)?,
        cmap.vertex_id_tx(
            t,
            if b1d1_old != NULL_DART_ID {
                b1d1_old
            } else if base_dart2 != NULL_DART_ID {
                base_dart2
            } else {
                abort(VertexInsertionError::UndefinedEdge)?
            },
        )?,
    );
    let (Some(v1), Some(v2)) = (cmap.read_vertex_tx(t, vid1)?, cmap.read_vertex_tx(t, vid2)?)
    else {
        abort(VertexInsertionError::UndefinedEdge)?
    };
    let seg = v2 - v1;

    // unsew current dart
    if b1d1_old != NULL_DART_ID {
        try_or_coerce!(cmap.unlink_tx::<1>(t, base_dart1), VertexInsertionError);
    }
    //
    if base_dart2 != NULL_DART_ID {
        try_or_coerce!(cmap.unlink_tx::<2>(t, base_dart1), VertexInsertionError);
    }
    // insert new vertices / darts on base_dart1's side
    let mut prev_d = base_dart1;
    for (&p, &new_d) in midpoint_vertices.iter().zip(darts_fh.iter()) {
        let new_v = v1 + seg * p;
        try_or_coerce!(cmap.link_tx::<1>(t, prev_d, new_d), VertexInsertionError);
        cmap.write_vertex_tx(t, new_d, new_v)?;
        prev_d = new_d;
    }
    try_or_coerce!(cmap.link_tx::<1>(t, prev_d, b1d1_old), VertexInsertionError);

    // if b2(base_dart1) is defined, insert vertices / darts on its side too
    if base_dart2 != NULL_DART_ID {
        let b1d2_old = cmap.beta_tx::<1>(t, base_dart2)?;
        if b1d2_old != NULL_DART_ID {
            try_or_coerce!(cmap.unlink_tx::<1>(t, base_dart2), VertexInsertionError);
        }
        let mut prev_d = base_dart2;
        for (d, new_d) in darts_fh.iter().rev().zip(darts_sh.iter()) {
            try_or_coerce!(cmap.link_tx::<2>(t, prev_d, *d), VertexInsertionError);
            try_or_coerce!(cmap.link_tx::<1>(t, prev_d, *new_d), VertexInsertionError);
            prev_d = *new_d;
        }
        if b1d2_old != NULL_DART_ID {
            try_or_coerce!(cmap.link_tx::<1>(t, prev_d, b1d2_old), VertexInsertionError);
        }
        try_or_coerce!(
            cmap.link_tx::<2>(t, prev_d, base_dart1),
            VertexInsertionError
        );
    }

    Ok(())
}
