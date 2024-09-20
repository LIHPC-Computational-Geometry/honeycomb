use honeycomb_core::cmap::CMap2;
use honeycomb_core::prelude::CMapBuilder;

// you can copy paste this function into the render example to see what the mesh looks like
// it contains:
// - one convex hexagon
// - one concave (still fannable) hexagon
// - one square
// - one triangle
// - one non-fannable n-gon
fn generate_map() -> CMap2<f64> {
    let mut cmap: CMap2<f64> = CMapBuilder::default().n_darts(28).build().unwrap();

    // topology
    cmap.one_link(1, 2);
    cmap.one_link(2, 3);
    cmap.one_link(3, 4);
    cmap.one_link(4, 5);
    cmap.one_link(5, 6);
    cmap.one_link(6, 1);

    cmap.one_link(7, 8);
    cmap.one_link(8, 9);
    cmap.one_link(9, 10);
    cmap.one_link(10, 11);
    cmap.one_link(11, 12);
    cmap.one_link(12, 7);

    cmap.one_link(13, 14);
    cmap.one_link(14, 15);
    cmap.one_link(15, 16);
    cmap.one_link(16, 13);

    cmap.one_link(17, 18);
    cmap.one_link(18, 19);
    cmap.one_link(19, 20);
    cmap.one_link(20, 21);
    cmap.one_link(21, 22);
    cmap.one_link(22, 23);
    cmap.one_link(23, 24);
    cmap.one_link(24, 25);
    cmap.one_link(25, 17);

    cmap.one_link(26, 27);
    cmap.one_link(27, 28);
    cmap.one_link(28, 26);

    cmap.two_link(3, 7);
    cmap.two_link(4, 13);
    cmap.two_link(10, 27);
    cmap.two_link(11, 26);
    cmap.two_link(12, 14);
    cmap.two_link(15, 17);
    cmap.two_link(18, 28);

    // geometry
    cmap.insert_vertex(1, (1.0, 0.0));
    cmap.insert_vertex(2, (2.0, 0.0));
    cmap.insert_vertex(3, (2.5, 0.5));
    cmap.insert_vertex(4, (2.0, 1.0));
    cmap.insert_vertex(5, (1.0, 1.0));
    cmap.insert_vertex(6, (0.5, 0.5));
    cmap.insert_vertex(9, (3.0, 1.0));
    cmap.insert_vertex(10, (3.0, 2.0));
    cmap.insert_vertex(11, (2.5, 1.0));
    cmap.insert_vertex(12, (2.0, 2.0));
    cmap.insert_vertex(16, (1.0, 2.0));
    cmap.insert_vertex(20, (3.0, 3.0));
    cmap.insert_vertex(21, (2.7, 3.0));
    cmap.insert_vertex(22, (2.7, 2.3));
    cmap.insert_vertex(23, (1.3, 2.3));
    cmap.insert_vertex(24, (1.3, 3.0));
    cmap.insert_vertex(25, (1.0, 3.0));

    cmap
}

#[cfg(test)]
fn fan_cell_convex() {
    // generate a map with all kinds of cell
    let map = generate_map();
}
