use honeycomb_core::{
    cmap::{CMap2, CMapBuilder, DartIdType, FaceIdType},
    stm::atomically_with_err,
};

use crate::triangulation::{TriangulateError, earclip_cell_countercw, fan_cell};

// you can copy paste this function into the render example to see what the mesh looks like
// it contains:
// - one convex hexagon
// - one concave (still fannable) hexagon
// - one square
// - one triangle
// - one non-fannable n-gon
fn generate_map() -> CMap2<f64> {
    let cmap: CMap2<f64> = CMapBuilder::<2>::from_n_darts(28).build().unwrap();

    // topology
    cmap.force_link::<1>(1, 2).unwrap();
    cmap.force_link::<1>(2, 3).unwrap();
    cmap.force_link::<1>(3, 4).unwrap();
    cmap.force_link::<1>(4, 5).unwrap();
    cmap.force_link::<1>(5, 6).unwrap();
    cmap.force_link::<1>(6, 1).unwrap();

    cmap.force_link::<1>(7, 8).unwrap();
    cmap.force_link::<1>(8, 9).unwrap();
    cmap.force_link::<1>(9, 10).unwrap();
    cmap.force_link::<1>(10, 11).unwrap();
    cmap.force_link::<1>(11, 12).unwrap();
    cmap.force_link::<1>(12, 7).unwrap();

    cmap.force_link::<1>(13, 14).unwrap();
    cmap.force_link::<1>(14, 15).unwrap();
    cmap.force_link::<1>(15, 16).unwrap();
    cmap.force_link::<1>(16, 13).unwrap();

    cmap.force_link::<1>(17, 18).unwrap();
    cmap.force_link::<1>(18, 19).unwrap();
    cmap.force_link::<1>(19, 20).unwrap();
    cmap.force_link::<1>(20, 21).unwrap();
    cmap.force_link::<1>(21, 22).unwrap();
    cmap.force_link::<1>(22, 23).unwrap();
    cmap.force_link::<1>(23, 24).unwrap();
    cmap.force_link::<1>(24, 25).unwrap();
    cmap.force_link::<1>(25, 17).unwrap();

    cmap.force_link::<1>(26, 27).unwrap();
    cmap.force_link::<1>(27, 28).unwrap();
    cmap.force_link::<1>(28, 26).unwrap();

    cmap.force_link::<2>(3, 7).unwrap();
    cmap.force_link::<2>(4, 13).unwrap();
    cmap.force_link::<2>(10, 27).unwrap();
    cmap.force_link::<2>(11, 26).unwrap();
    cmap.force_link::<2>(12, 14).unwrap();
    cmap.force_link::<2>(15, 17).unwrap();
    cmap.force_link::<2>(18, 28).unwrap();

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

#[test]
fn fan_cells() {
    // generate a map with all kinds of cell
    let mut map = generate_map();
    // we know these by construction
    let hex1: FaceIdType = 1;
    let hex2: FaceIdType = 7;
    let squ: FaceIdType = 13;
    let nop: FaceIdType = 17;
    let tri: FaceIdType = 26;

    // the hex will be
    let nd = map.allocate_used_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert!(atomically_with_err(|t| fan_cell(t, &map, hex1, &new_darts)).is_ok());

    assert_eq!(map.i_cell::<2>(hex1 as DartIdType).count(), 3);
    assert_eq!(map.i_cell::<2>(3).count(), 3);
    assert_eq!(map.i_cell::<2>(4).count(), 3);
    assert_eq!(map.i_cell::<2>(5).count(), 3);

    // the hex will be
    let nd = map.allocate_used_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert!(atomically_with_err(|t| fan_cell(t, &map, hex2, &new_darts)).is_ok());

    assert_eq!(map.i_cell::<2>(hex2 as DartIdType).count(), 3);
    assert_eq!(map.i_cell::<2>(8).count(), 3);
    assert_eq!(map.i_cell::<2>(10).count(), 3);
    assert_eq!(map.i_cell::<2>(11).count(), 3);

    // the square will be split in two
    let nd = map.allocate_used_darts(2);
    let new_darts = (nd..nd + 2).collect::<Vec<_>>();
    assert!(atomically_with_err(|t| fan_cell(t, &map, squ, &new_darts)).is_ok());

    assert_eq!(map.i_cell::<2>(squ as DartIdType).count(), 3);
    assert_eq!(map.i_cell::<2>(15).count(), 3);

    // this will be a no-op since the polygon isn't fannable
    let nd = map.allocate_used_darts(12);
    let new_darts = (nd..nd + 12).collect::<Vec<_>>();
    assert_eq!(
        atomically_with_err(|t| fan_cell(t, &map, nop, &new_darts)),
        Err(TriangulateError::NonFannable)
    );

    assert_eq!(map.i_cell::<2>(nop as DartIdType).count(), 9); // unchanged

    assert_eq!(
        atomically_with_err(|t| fan_cell(t, &map, tri, &[])),
        Err(TriangulateError::AlreadyTriangulated)
    );

    assert_eq!(map.i_cell::<2>(tri as DartIdType).count(), 3); // unchanged
}

#[test]
fn earclip_cells() {
    // generate a map with all kinds of cell
    let mut map = generate_map();
    // we know these by construction
    let hex1: FaceIdType = 1;
    let hex2: FaceIdType = 7;
    let squ: FaceIdType = 13;
    let smh: FaceIdType = 17;
    let tri: FaceIdType = 26;

    // the hex will be split in 4
    let nd = map.allocate_used_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert_eq!(
        atomically_with_err(|t| earclip_cell_countercw(t, &map, hex1, &new_darts)),
        Ok(())
    );

    assert_eq!(map.i_cell::<2>(hex1 as DartIdType).count(), 3);
    assert_eq!(map.i_cell::<2>(3).count(), 3);
    assert_eq!(map.i_cell::<2>(4).count(), 3);
    assert_eq!(map.i_cell::<2>(5).count(), 3);

    // the hex will be split in 4
    let nd = map.allocate_used_darts(6);
    let new_darts = (nd..nd + 6).collect::<Vec<_>>();
    assert!(atomically_with_err(|t| earclip_cell_countercw(t, &map, hex2, &new_darts)).is_ok());

    assert_eq!(map.i_cell::<2>(hex2 as DartIdType).count(), 3);
    assert_eq!(map.i_cell::<2>(8).count(), 3);
    assert_eq!(map.i_cell::<2>(10).count(), 3);
    assert_eq!(map.i_cell::<2>(11).count(), 3);

    // the square will be split in 2
    let nd = map.allocate_used_darts(2);
    let new_darts = (nd..nd + 2).collect::<Vec<_>>();
    assert!(atomically_with_err(|t| earclip_cell_countercw(t, &map, squ, &new_darts)).is_ok());

    assert_eq!(map.i_cell::<2>(squ as DartIdType).count(), 3);
    assert_eq!(map.i_cell::<2>(15).count(), 3);

    // 9-gon is split in 7
    let nd = map.allocate_used_darts(12);
    let new_darts = (nd..nd + 12).collect::<Vec<_>>();
    assert!(atomically_with_err(|t| earclip_cell_countercw(t, &map, smh, &new_darts)).is_ok());

    assert_eq!(map.i_cell::<2>(smh as DartIdType).count(), 3);
    assert_eq!(map.i_cell::<2>(18).count(), 3);
    assert_eq!(map.i_cell::<2>(19).count(), 3);
    assert_eq!(map.i_cell::<2>(21).count(), 3);
    assert_eq!(map.i_cell::<2>(22).count(), 3);
    assert_eq!(map.i_cell::<2>(23).count(), 3);
    assert_eq!(map.i_cell::<2>(24).count(), 3);

    assert_eq!(
        atomically_with_err(|t| earclip_cell_countercw(t, &map, tri, &[])),
        Err(TriangulateError::AlreadyTriangulated)
    );

    assert_eq!(map.i_cell::<2>(tri as DartIdType).count(), 3); // unchanged
}
