use honeycomb_core::{
    cmap::{CMap2, CMap3, DartIdType, NULL_DART_ID, OrbitPolicy, VertexIdType, VolumeIdType},
    geometry::{CoordsFloat, Vertex2, Vertex3},
    stm::{StmClosureResult, Transaction, retry},
};
use nalgebra::Matrix3;
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
    if let Some(new_v) = map.read_vertex(t, vid)? {
        let mut tmp: SmallVec<DartIdType, 10> = SmallVec::new();
        for d in map.orbit_tx(t, OrbitPolicy::Vertex, vid) {
            tmp.push(d?);
        }

        let ref_crossp = {
            let d = tmp[0];
            let b1d = map.beta_tx::<1>(t, d)?;
            let b1b1d = map.beta_tx::<1>(t, b1d)?;
            let vid1 = map.vertex_id_tx(t, b1d)?;
            let vid2 = map.vertex_id_tx(t, b1b1d)?;
            let v1 = if let Some(v) = map.read_vertex(t, vid1)? {
                v
            } else {
                retry()?
            };
            let v2 = if let Some(v) = map.read_vertex(t, vid2)? {
                v
            } else {
                retry()?
            };

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
            let v1 = if let Some(v) = map.read_vertex(t, vid1)? {
                v
            } else {
                retry()?
            };
            let v2 = if let Some(v) = map.read_vertex(t, vid2)? {
                v
            } else {
                retry()?
            };

            let crossp = Vertex2::cross_product_from_vertices(&new_v, &v1, &v2);

            if ref_sign != crossp.signum() || crossp.is_zero() {
                return Ok(false);
            }
        }
    } else {
        retry()?;
    }

    Ok(true)
}

#[rustfmt::skip]
pub fn compute_tet_orientation<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    (d1, d2, d3): (DartIdType, DartIdType, DartIdType),
    p: Vertex3<T>,
) -> StmClosureResult<f64> {
    let v1 = {
        let vid = map.vertex_id_tx(t, d1).unwrap();
        map.read_vertex(t, vid)?.unwrap()
    };
    let v2 = {
        let vid = map.vertex_id_tx(t, d2).unwrap();
        map.read_vertex(t, vid)?.unwrap()
    };
    let v3 = {
        let vid = map.vertex_id_tx(t, d3).unwrap();
        map.read_vertex(t, vid)?.unwrap()
    };

    let c1 = v1 - p;
    let c2 = v2 - p;
    let c3 = v3 - p;

    Ok(Matrix3::from_column_slice(&[
        c1.x().to_f64().unwrap(), c1.y().to_f64().unwrap(), c1.z().to_f64().unwrap(), 
        c2.x().to_f64().unwrap(), c2.y().to_f64().unwrap(), c2.z().to_f64().unwrap(), 
        c3.x().to_f64().unwrap(), c3.y().to_f64().unwrap(), c3.z().to_f64().unwrap(), 
    ]).determinant())
}

pub fn locate_containing_tet<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap3<T>,
    start: VolumeIdType,
    p: Vertex3<T>,
) -> StmClosureResult<Option<VolumeIdType>> {
    fn locate_next_tet<T: CoordsFloat>(
        t: &mut Transaction,
        map: &CMap3<T>,
        d: DartIdType,
        p: Vertex3<T>,
    ) -> StmClosureResult<Option<DartIdType>> {
        let face_darts = [
            d as DartIdType,
            { map.beta_tx::<2>(t, d)? },
            {
                let b1 = map.beta_tx::<1>(t, d)?;
                map.beta_tx::<2>(t, b1)?
            },
            {
                let b0 = map.beta_tx::<0>(t, d)?;
                map.beta_tx::<2>(t, b0)?
            },
        ];

        // TODO: does caching vids and vertices values improve perf?
        for d in face_darts {
            let b1 = map.beta_tx::<1>(t, d)?;
            let b0 = map.beta_tx::<0>(t, d)?;

            if compute_tet_orientation(t, map, (d, b1, b0), p)? < 0.0 {
                return Ok(Some(map.beta_tx::<3>(t, d)?));
            }
        }
        Ok(None)
    }

    let mut dart = start as DartIdType;

    loop {
        if let Some(next_dart) = locate_next_tet(t, map, dart, p)? {
            dart = next_dart;
            // point is outside or across a gap in the mesh
            if dart == NULL_DART_ID {
                // it is possible to look for another path, but it requires a more complex condition
                // than "just follow the first neg volume direction"
                return Ok(None);
            }
        } else {
            return Ok(Some(map.volume_id_tx(t, dart)?));
        }
    }
}
