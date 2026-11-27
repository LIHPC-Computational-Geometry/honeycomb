use honeycomb::prelude::*;

pub fn cut_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    e: EdgeIdType,
    new_darts: [DartIdType; 6],
) -> Result<(), SewError> {
    let (d1, d4) = (e as DartIdType, map.beta::<2>(e as DartIdType));
    assert_ne!(d4, NULL_DART_ID);
    let (d2, d3) = (map.beta::<1>(d1), map.beta::<0>(d1));
    let (d5, d6) = (map.beta::<1>(d4), map.beta::<0>(d4));

    // prepare new darts
    let [d7, d8, d9, d10, d11, d12] = new_darts;
    let (vid1, vid2) = (map.vertex_id(d1), map.vertex_id(d4));
    let (v1, v2) = (
        map.read_vertex(vid1)
            .expect("E: no associated coordinates to vertex"),
        map.read_vertex(vid2)
            .expect("E: no associated coordinates to vertex"),
    );
    let v_new = Vertex2::average(&v1, &v2);
    map.link::<1>(d8, d9)?;
    map.link::<1>(d10, d11)?;
    map.link::<2>(d7, d8)?;
    map.link::<2>(d10, d11)?;
    map.write_vertex(map.vertex_id(d7), v_new);
    map.write_vertex(map.vertex_id(d10), v_new);

    // cut the edge
    map.unsew::<2>(d1)?;
    map.unsew::<1>(d1)?;
    map.unsew::<1>(d2)?;
    map.unsew::<1>(d4)?;
    map.unsew::<1>(d5)?;

    map.sew::<1>(d1, d7)?;
    map.sew::<1>(d7, d3)?;
    map.sew::<1>(d2, d8)?;
    map.sew::<1>(d9, d2)?;
    map.sew::<1>(d4, d10)?;
    map.sew::<1>(d10, d6)?;
    map.sew::<1>(d5, d11)?;
    map.sew::<1>(d12, d5)?;

    map.sew::<2>(d1, d12)?;
    map.sew::<2>(d4, d9)?;

    Ok(())
}
