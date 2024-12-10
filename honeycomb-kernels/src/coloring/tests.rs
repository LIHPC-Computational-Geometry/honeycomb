use honeycomb_core::cmap::{CMap2, CMapBuilder, Orbit2, OrbitPolicy};
use itertools::Itertools;

use crate::coloring::{color_dsatur, Color};

// same map as the triangulation tests
// you can copy paste it into the render example to visualize it
fn generate_map() -> CMap2<f64> {
    let cmap: CMap2<f64> = CMapBuilder::default().n_darts(28).build().unwrap();

    // topology
    cmap.force_one_link(1, 2);
    cmap.force_one_link(2, 3);
    cmap.force_one_link(3, 4);
    cmap.force_one_link(4, 5);
    cmap.force_one_link(5, 6);
    cmap.force_one_link(6, 1);

    cmap.force_one_link(7, 8);
    cmap.force_one_link(8, 9);
    cmap.force_one_link(9, 10);
    cmap.force_one_link(10, 11);
    cmap.force_one_link(11, 12);
    cmap.force_one_link(12, 7);

    cmap.force_one_link(13, 14);
    cmap.force_one_link(14, 15);
    cmap.force_one_link(15, 16);
    cmap.force_one_link(16, 13);

    cmap.force_one_link(17, 18);
    cmap.force_one_link(18, 19);
    cmap.force_one_link(19, 20);
    cmap.force_one_link(20, 21);
    cmap.force_one_link(21, 22);
    cmap.force_one_link(22, 23);
    cmap.force_one_link(23, 24);
    cmap.force_one_link(24, 25);
    cmap.force_one_link(25, 17);

    cmap.force_one_link(26, 27);
    cmap.force_one_link(27, 28);
    cmap.force_one_link(28, 26);

    cmap.force_two_link(3, 7);
    cmap.force_two_link(4, 13);
    cmap.force_two_link(10, 27);
    cmap.force_two_link(11, 26);
    cmap.force_two_link(12, 14);
    cmap.force_two_link(15, 17);
    cmap.force_two_link(18, 28);

    // geometry
    cmap.force_write_vertex(1, (1.0, 0.0));
    cmap.force_write_vertex(2, (2.0, 0.0));
    cmap.force_write_vertex(3, (2.5, 0.5));
    cmap.force_write_vertex(4, (2.0, 1.0));
    cmap.force_write_vertex(5, (1.0, 1.0));
    cmap.force_write_vertex(6, (0.5, 0.5));
    cmap.force_write_vertex(9, (3.0, 1.0));
    cmap.force_write_vertex(10, (3.0, 2.0));
    cmap.force_write_vertex(11, (2.5, 1.0));
    cmap.force_write_vertex(12, (2.0, 2.0));
    cmap.force_write_vertex(16, (1.0, 2.0));
    cmap.force_write_vertex(20, (3.0, 3.0));
    cmap.force_write_vertex(21, (2.7, 3.0));
    cmap.force_write_vertex(22, (2.7, 2.3));
    cmap.force_write_vertex(23, (1.3, 2.3));
    cmap.force_write_vertex(24, (1.3, 3.0));
    cmap.force_write_vertex(25, (1.0, 3.0));

    cmap
}

// test many different maps for basic properties of the result
#[test]
fn dsatur_invariants() {
    {
        let mut map: CMap2<f64> = CMapBuilder::unit_grid(8).build().unwrap();
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
                Orbit2::new(&map, OrbitPolicy::Vertex, *id)
                    .flat_map(|d| {
                        [
                            map.vertex_id(map.beta::<1>(d)),
                            // needed when both nodes are on the boundary
                            map.vertex_id(map.beta::<0>(d)),
                        ]
                        .into_iter()
                    })
                    .unique()
                    .map(|v| map.force_read_attribute::<Color>(v).unwrap())
            ))
            .all(|(c, mut cns)| cns.all(|cn| cn != c)));
    }

    {
        let mut map: CMap2<f64> = CMapBuilder::unit_triangles(8).build().unwrap();
        let colors = 0..=color_dsatur(&mut map);
        let vertices = map.fetch_vertices().identifiers.clone();

        for &v in &vertices {
            println!(
                "Vertex {v} has color {:?}",
                map.force_read_attribute::<Color>(v).unwrap()
            );
        }

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
                Orbit2::new(&map, OrbitPolicy::Vertex, *id)
                    .flat_map(|d| {
                        [
                            map.vertex_id(map.beta::<1>(d)),
                            // needed when both nodes are on the boundary
                            map.vertex_id(map.beta::<0>(d)),
                        ]
                        .into_iter()
                    })
                    .unique()
                    .map(|v| map.force_read_attribute::<Color>(v).unwrap())
            ))
            .all(|(c, mut cns)| cns.all(|cn| cn != c)));
    }

    {
        let mut map: CMap2<f64> = generate_map();
        let colors = 0..=color_dsatur(&mut map);
        let vertices = map.fetch_vertices().identifiers.clone();

        for &v in &vertices {
            println!(
                "Vertex {v} has color {:?}",
                map.force_read_attribute::<Color>(v).unwrap()
            );
        }

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
                Orbit2::new(&map, OrbitPolicy::Vertex, *id)
                    .flat_map(|d| {
                        [
                            map.vertex_id(map.beta::<1>(d)),
                            // needed when both nodes are on the boundary
                            map.vertex_id(map.beta::<0>(d)),
                        ]
                        .into_iter()
                    })
                    .unique()
                    .map(|v| map.force_read_attribute::<Color>(v).unwrap())
            ))
            .all(|(c, mut cns)| cns.all(|cn| cn != c)));
    }
}
