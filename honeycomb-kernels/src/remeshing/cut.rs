use honeycomb_core::{
    cmap::{CMap2, DartIdType, EdgeIdType, SewError},
    geometry::{CoordsFloat, Vertex2},
    stm::{Transaction, TransactionClosureResult, retry, try_or_coerce},
};

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
