use crate::triangulation::{earclip_cell, fan_cell, TriangulateError};
use honeycomb_core::cmap::{CMap2, FaceIdentifier};
use honeycomb_core::common::DartIdentifier;
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

#[test]
fn fan_cells() {
    // generate a map with all kinds of cell
    let mut map = generate_map();
    // we know these by construction
    let hex1: FaceIdentifier = 1;
    let hex2: FaceIdentifier = 7;
    let squ: FaceIdentifier = 13;
    let nop: FaceIdentifier = 17;
    let tri: FaceIdentifier = 26;

    // the hex will be
    let nd = map.add_free_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert!(fan_cell(&mut map, hex1, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex1 as DartIdentifier).count(), 3);
    assert_eq!(map.i_cell::<2>(3).count(), 3);
    assert_eq!(map.i_cell::<2>(4).count(), 3);
    assert_eq!(map.i_cell::<2>(5).count(), 3);

    // the hex will be
    let nd = map.add_free_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert!(fan_cell(&mut map, hex2, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex2 as DartIdentifier).count(), 3);
    assert_eq!(map.i_cell::<2>(8).count(), 3);
    assert_eq!(map.i_cell::<2>(10).count(), 3);
    assert_eq!(map.i_cell::<2>(11).count(), 3);

    // the square will be split in two
    let nd = map.add_free_darts(2);
    let new_darts = (nd..nd + 2).collect::<Vec<_>>();
    assert!(fan_cell(&mut map, squ, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(squ as DartIdentifier).count(), 3);
    assert_eq!(map.i_cell::<2>(15).count(), 3);

    // this will be a no-op since the polygon isn't fannable
    let nd = map.add_free_darts(12);
    let new_darts = (nd..nd + 12).collect::<Vec<_>>();
    assert_eq!(
        fan_cell(&mut map, nop, &new_darts),
        Err(TriangulateError::NonFannable)
    );

    assert_eq!(map.i_cell::<2>(nop as DartIdentifier).count(), 9); // unchanged

    assert_eq!(
        fan_cell(&mut map, tri, &[]),
        Err(TriangulateError::AlreadyTriangulated)
    );

    assert_eq!(map.i_cell::<2>(tri as DartIdentifier).count(), 3); // unchanged
}

#[test]
fn earclip_cells() {
    // generate a map with all kinds of cell
    let mut map = generate_map();
    // we know these by construction
    let hex1: FaceIdentifier = 1;
    let hex2: FaceIdentifier = 7;
    let squ: FaceIdentifier = 13;
    let smh: FaceIdentifier = 17;
    let tri: FaceIdentifier = 26;

    // the hex will be split in 4
    let nd = map.add_free_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, hex1, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex1 as DartIdentifier).count(), 3);
    assert_eq!(map.i_cell::<2>(3).count(), 3);
    assert_eq!(map.i_cell::<2>(4).count(), 3);
    assert_eq!(map.i_cell::<2>(5).count(), 3);

    // the hex will be split in 4
    let nd = map.add_free_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, hex2, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex2 as DartIdentifier).count(), 3);
    assert_eq!(map.i_cell::<2>(8).count(), 3);
    assert_eq!(map.i_cell::<2>(10).count(), 3);
    assert_eq!(map.i_cell::<2>(11).count(), 3);

    // the square will be split in 2
    let nd = map.add_free_darts(2);
    let new_darts = (nd..nd + 2).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, squ, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(squ as DartIdentifier).count(), 3);
    assert_eq!(map.i_cell::<2>(15).count(), 3);

    // 9-gon is split in 7
    let nd = map.add_free_darts(12);
    let new_darts = (nd..nd + 12).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, smh, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(smh as DartIdentifier).count(), 3);
    assert_eq!(map.i_cell::<2>(18).count(), 3);
    assert_eq!(map.i_cell::<2>(19).count(), 3);
    assert_eq!(map.i_cell::<2>(21).count(), 3);
    assert_eq!(map.i_cell::<2>(22).count(), 3);
    assert_eq!(map.i_cell::<2>(23).count(), 3);
    assert_eq!(map.i_cell::<2>(24).count(), 3);

    assert_eq!(
        earclip_cell(&mut map, tri, &[]),
        Err(TriangulateError::AlreadyTriangulated)
    );

    assert_eq!(map.i_cell::<2>(tri as DartIdentifier).count(), 3); // unchanged
}
