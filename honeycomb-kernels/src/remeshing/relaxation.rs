use honeycomb_core::{
    cmap::{CMap2, VertexIdType},
    geometry::{CoordsFloat, Vector2},
    stm::{StmClosureResult, Transaction},
};

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
    neighbor_based_smooth(t, map, vid, others, T::one())
}

/// Generic neighbor-based vertex smoothing function.
///
/// This function smooths the vertex position by moving it toward the average of its neighbors'
/// positions weighted by lambda.
///
/// Note that it is up to the user to provide a correct list of neighbor IDs, and "acceptable"
/// lambda parameter. For example:
///
/// - `lambda == 1` nullifies the influence of the original vertex position,
/// - `0 < lambda < 1` results in a Laplacian smoothing.
///
/// # Arguments
///
/// - `t: &mut Transaction` -- Associated transaction.
/// - `map: &mut CMap2` -- Edited map.
/// - `vid: VertexIdType` -- Vertex to move.
/// - `neighbors_id: &[VertexIdType]` -- List of vertex to compute the average from.
/// - `lambda: T` -- Coefficient weighting the applied offset.
///
/// # Errors
///
/// This function will abort and raise an error if the transaction cannot be completed.
///
/// # Panics
///
/// This function may panic if one vertex in the `neighbors_id` list has no associated coordinates.
#[inline]
pub fn neighbor_based_smooth<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    vid: VertexIdType,
    neighbors_id: &[VertexIdType],
    lambda: T,
) -> StmClosureResult<()> {
    let p = map
        .read_vertex(t, vid)?
        .expect("E: no coordinates associated to vertex ID");

    let n = neighbors_id.len();
    let mut neighbors: smallvec::SmallVec<_, 16> = smallvec::SmallVec::with_capacity(n);
    for &nid in neighbors_id {
        neighbors.push(
            map.read_vertex(t, nid)?
                .expect("E: no coordinates associated to vertex ID"),
        );
    }

    let delta = neighbors
        .into_iter()
        .map(|v| v - p)
        .fold(Vector2::default(), |a, b| a + b)
        * lambda
        / T::from(n).unwrap();

    map.write_vertex(t, vid, p + delta)?;

    Ok(())
}
