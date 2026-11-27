use honeycomb::prelude::*;

pub fn cut_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    e: EdgeIdType,
    new_darts: [DartIdType; 6],
) -> Result<(), SewError> {
    /* ... */
    Ok(())
}

pub fn main() {
    let mut map: CMap2<f64> = /* ... */;

    // collect the edges to cut first
    let edges: Vec<EdgeIdType> = map
        .iter_edges()
        .filter(|e| {
            // keep only inner edges
            map.beta::<2>(e as DartIdType) != NULL_DART_ID
        })
        .collect();


    for e in edges {
        let first_new = map.allocate_used_darts(6);
        let nds: [DartIdType; 6] = std::array::from_fn(|i| first_new + i as DartIdType);
        cut_edge(&map, e, nds).unwrap();
    }
}
