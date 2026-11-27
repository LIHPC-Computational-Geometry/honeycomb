use honeycomb::prelude::*;

pub fn smooth_vertex<T: CoordsFloat>(map: &CMap2<T>, vid: VertexIdType) {
    /* ... */
}

pub fn main() {
    let map: CMap2<f64> = /* ... */;

    //  fetch inner vertices
    let interior: Vec<VertexIdType> = map
        .iter_vertices()
        .filter(|&v| {
            // boundary vertices have a null beta_2 image in their orbit
            map.orbit(OrbitPolicy::Vertex, v as DartIdType)
                .all(|d| map.beta::<2>(d) != NULL_DART_ID)
        })
        .collect();

    // do 10 rounds of smoothing
    for _ in 0..10 {
        for &vid in &interior {
            smooth_vertex(&map, vid);
        }
    }
}
