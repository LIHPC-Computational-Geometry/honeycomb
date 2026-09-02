use honeycomb_core::{
    cmap::{CMap2, CMap3, VertexIdType},
    geometry::{CoordsFloat, Vector2, Vector3},
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
        .read_vertex_tx(t, vid)?
        .expect("E: no coordinates associated to vertex ID");

    let n = neighbors_id.len();
    let mut neighbors: smallvec::SmallVec<_, 16> = smallvec::SmallVec::with_capacity(n);
    for &nid in neighbors_id {
        neighbors.push(
            map.read_vertex_tx(t, nid)?
                .expect("E: no coordinates associated to vertex ID"),
        );
    }

    let delta = neighbors
        .into_iter()
        .map(|v| v - p)
        .fold(Vector2::default(), |a, b| a + b)
        * lambda
        / T::from(n).unwrap();

    map.write_vertex_tx(t, vid, p + delta)?;

    Ok(())
}

/// Move a 3-map vertex to the average of the supplied neighboring vertices.
///
/// # Errors
///
/// This function will abort and raise an error if the transaction cannot be completed.
///
/// # Panics
///
/// This function may panic if one of the vertices has no associated coordinates.
#[inline]
pub fn move_vertex_to_average_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    vid: VertexIdType,
    others: &[VertexIdType],
) -> StmClosureResult<()> {
    neighbor_based_smooth_3d(t, map, vid, others, T::one())
}

/// Smooth a 3-map vertex toward the average of the supplied neighboring vertices.
///
/// # Errors
///
/// This function will abort and raise an error if the transaction cannot be completed.
///
/// # Panics
///
/// This function may panic if one of the vertices has no associated coordinates.
#[inline]
pub fn neighbor_based_smooth_3d<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    vid: VertexIdType,
    neighbors_id: &[VertexIdType],
    lambda: T,
) -> StmClosureResult<()> {
    let p = map
        .read_vertex_tx(t, vid)?
        .expect("E: no coordinates associated to vertex ID");

    let n = neighbors_id.len();
    let mut neighbors: smallvec::SmallVec<_, 16> = smallvec::SmallVec::with_capacity(n);
    for &nid in neighbors_id {
        neighbors.push(
            map.read_vertex_tx(t, nid)?
                .expect("E: no coordinates associated to vertex ID"),
        );
    }

    let delta = neighbors
        .into_iter()
        .map(|v| v - p)
        .fold(Vector3::default(), |a, b| a + b)
        * lambda
        / T::from(n).unwrap();

    map.write_vertex_tx(t, vid, p + delta)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use honeycomb_core::{
        cmap::{CMap3, CMapBuilder},
        geometry::Vertex3,
        stm::atomically,
    };

    use super::{move_vertex_to_average_3d, neighbor_based_smooth_3d};

    #[test]
    fn smooth_vertices_3d() {
        let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(3).build().unwrap();
        map.set_vertex(1, Vertex3(0.0, 0.0, 0.0));
        map.set_vertex(2, Vertex3(2.0, 0.0, 0.0));
        map.set_vertex(3, Vertex3(0.0, 2.0, 2.0));

        atomically(|transaction| move_vertex_to_average_3d(transaction, &map, 1, &[2, 3]));
        assert_eq!(map.read_vertex(1), Some(Vertex3(1.0, 1.0, 1.0)));

        map.set_vertex(1, Vertex3(0.0, 0.0, 0.0));
        atomically(|transaction| neighbor_based_smooth_3d(transaction, &map, 1, &[2, 3], 0.5));
        assert_eq!(map.read_vertex(1), Some(Vertex3(0.5, 0.5, 0.5)));
    }
}
