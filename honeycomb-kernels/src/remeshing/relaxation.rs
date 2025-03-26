use honeycomb_core::{
    cmap::{CMap2, VertexIdType},
    geometry::{CoordsFloat, Vertex2},
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
