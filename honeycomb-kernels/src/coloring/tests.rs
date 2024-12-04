use honeycomb_core::cmap::{CMap2, CMapBuilder, Orbit2, OrbitPolicy};

use crate::coloring::{color_dsatur, Color};

// test many different maps for basic properties of the result
#[test]
fn dsatur_invariants() {
    let mut map: CMap2<f64> = CMapBuilder::unit_grid(64).build().unwrap();
    let colors = 0..=color_dsatur(&mut map);
    let vertices = map.fetch_vertices().identifiers.clone();

    // all vertices are colored, colors being in (0..=cmax)
    assert!(vertices
        .iter()
        .map(|id| map.force_read_attribute::<Color>(*id))
        .all(|c| c.is_some_and(|Color(val)| colors.contains(&val))));
    // for all vertices v, v has no neighbor with the same color
    assert!(vertices
        .iter()
        .map(|id| (
            map.force_read_attribute::<Color>(*id).unwrap(),
            Orbit2::new(&map, OrbitPolicy::Vertex, *id).map(|d| map
                .force_read_attribute::<Color>(map.vertex_id(map.beta::<1>(d)))
                .unwrap())
        ))
        .all(|(c, mut cns)| cns.all(|cn| cn != c)));
}
