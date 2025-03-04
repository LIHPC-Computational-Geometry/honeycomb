//!

use honeycomb_core::{
    cmap::{CMap2, DartIdType, EdgeIdType, LinkError, VertexIdType},
    geometry::{CoordsFloat, Vertex2},
    stm::{StmClosureResult, StmError, Transaction, TransactionClosureResult, retry},
};

// -- vertex relaxation

/// Move a vertex to the average of the others' values.
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
///
#[inline]
pub fn cut_outer_edge<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
    [nd1, nd2, nd3]: [DartIdType; 3],
) -> TransactionClosureResult<(), LinkError> {
    // unfallible
    map.link::<2>(t, nd1, nd2)?;
    map.link::<1>(t, nd2, nd3)?;

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

    map.unsew::<1>(t, ld).map_err(|_| StmError::Retry)?;
    map.unsew::<1>(t, b1ld).map_err(|_| StmError::Retry)?;

    map.sew::<1>(t, ld, nd1).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, nd1, b0ld).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, nd3, b1ld).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, b1ld, nd2).map_err(|_| StmError::Retry)?;

    Ok(())
}

/// Cut an edge in half and build triangles from the new vertex.
///
///
#[inline]
pub fn cut_inner_edge<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    e: EdgeIdType,
    [nd1, nd2, nd3, nd4, nd5, nd6]: [DartIdType; 6],
) -> TransactionClosureResult<(), LinkError> {
    // unfallible
    map.link::<2>(t, nd1, nd2)?;
    map.link::<1>(t, nd2, nd3)?;
    map.link::<2>(t, nd4, nd5)?;
    map.link::<1>(t, nd5, nd6)?;

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

    map.unsew::<2>(t, ld).map_err(|_| StmError::Retry)?;
    map.unsew::<1>(t, ld).map_err(|_| StmError::Retry)?;
    map.unsew::<1>(t, b1ld).map_err(|_| StmError::Retry)?;
    map.unsew::<1>(t, rd).map_err(|_| StmError::Retry)?;
    map.unsew::<1>(t, b1rd).map_err(|_| StmError::Retry)?;

    map.sew::<2>(t, ld, nd6).map_err(|_| StmError::Retry)?;
    map.sew::<2>(t, rd, nd3).map_err(|_| StmError::Retry)?;

    map.sew::<1>(t, ld, nd1).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, nd1, b0ld).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, nd3, b1ld).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, b1ld, nd2).map_err(|_| StmError::Retry)?;

    map.sew::<1>(t, rd, nd4).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, nd4, b0rd).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, nd6, b1rd).map_err(|_| StmError::Retry)?;
    map.sew::<1>(t, b1rd, nd5).map_err(|_| StmError::Retry)?;

    Ok(())
}
