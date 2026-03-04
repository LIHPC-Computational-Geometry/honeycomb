use honeycomb_core::{
    cmap::{CMap2, DartIdType, OrbitPolicy, VertexIdType},
    geometry::{CoordsFloat, Vertex2},
    stm::{StmClosureResult, Transaction, unwrap_or_retry},
};
use smallvec::SmallVec;

/// Check if all faces incident to the vertex have the same orientation.
///
/// Note that this function expects the incident faces to be triangles.
///
/// # Errors
///
/// This method is meant to be called in a context where the returned `Result` is used to
/// validate the transaction passed as argument. Errors should not be processed manually,
/// only processed via the `?` operator.
pub fn is_orbit_orientation_consistent<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    vid: VertexIdType,
) -> StmClosureResult<bool> {
    let mut tmp: SmallVec<DartIdType, 10> = SmallVec::new();
    for d in map.orbit_tx(t, OrbitPolicy::Vertex, vid) {
        tmp.push(d?);
    }
    let new_v = unwrap_or_retry(map.read_vertex_tx(t, vid)?)?;

    let ref_crossp = {
        let d = tmp[0];
        let b1d = map.beta_tx::<1>(t, d)?;
        let b1b1d = map.beta_tx::<1>(t, b1d)?;
        let vid1 = map.vertex_id_tx(t, b1d)?;
        let vid2 = map.vertex_id_tx(t, b1b1d)?;
        let v1 = unwrap_or_retry(map.read_vertex_tx(t, vid1)?)?;
        let v2 = unwrap_or_retry(map.read_vertex_tx(t, vid2)?)?;

        Vertex2::cross_product_from_vertices(&new_v, &v1, &v2)
    };
    if ref_crossp.is_zero() {
        return Ok(false);
    }

    let ref_sign = ref_crossp.signum();
    for &d in &tmp[1..] {
        let b1d = map.beta_tx::<1>(t, d)?;
        let b1b1d = map.beta_tx::<1>(t, b1d)?;
        let vid1 = map.vertex_id_tx(t, b1d)?;
        let vid2 = map.vertex_id_tx(t, b1b1d)?;
        let v1 = unwrap_or_retry(map.read_vertex_tx(t, vid1)?)?;
        let v2 = unwrap_or_retry(map.read_vertex_tx(t, vid2)?)?;

        let crossp = Vertex2::cross_product_from_vertices(&new_v, &v1, &v2);

        if ref_sign != crossp.signum() || crossp.is_zero() {
            return Ok(false);
        }
    }

    Ok(true)
}
