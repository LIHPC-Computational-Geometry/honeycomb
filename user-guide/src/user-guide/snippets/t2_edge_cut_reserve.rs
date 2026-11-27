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
    // let mut map: CMap2<f64> = /* ... */;

    // collect the edges to cut first
    let edges: Vec<EdgeIdType> = map
        .iter_edges()
        .filter(|e| {
            // keep only inner edges
            map.beta::<2>(e as DartIdType) != NULL_DART_ID
        })
        .collect();

    // allocate all the darts that will be used first
    map.allocate_unused_darts(6 * edges.len());

    for e in edges {
        // `reserve_darts` returns a vector which we convert to an array of known size
        // the routine also marks the returned dart as used
        let nds: [DartIdType; 6] = map.reserve_darts(6).unwrap().try_into().unwrap();
        cut_edge(&map, e, nds).unwrap();
    }
}
