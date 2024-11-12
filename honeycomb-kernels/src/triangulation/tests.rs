use crate::triangulation::{earclip_cell, fan_cell, TriangulateError};
use honeycomb_core::cmap::{CMap2, DartId, FaceId, VertexId};
use honeycomb_core::prelude::CMapBuilder;

// you can copy paste this function into the render example to see what the mesh looks like
// it contains:
// - one convex hexagon
// - one concave (still fannable) hexagon
// - one square
// - one triangle
// - one non-fannable n-gon
fn generate_map() -> CMap2<f64> {
    let cmap: CMap2<f64> = CMapBuilder::default().n_darts(28).build().unwrap();

    // topology
    cmap.one_link(DartId(1), DartId(2));
    cmap.one_link(DartId(2), DartId(3));
    cmap.one_link(DartId(3), DartId(4));
    cmap.one_link(DartId(4), DartId(5));
    cmap.one_link(DartId(5), DartId(6));
    cmap.one_link(DartId(6), DartId(1));

    cmap.one_link(DartId(7), DartId(8));
    cmap.one_link(DartId(8), DartId(9));
    cmap.one_link(DartId(9), DartId(10));
    cmap.one_link(DartId(10), DartId(11));
    cmap.one_link(DartId(11), DartId(12));
    cmap.one_link(DartId(12), DartId(7));

    cmap.one_link(DartId(13), DartId(14));
    cmap.one_link(DartId(14), DartId(15));
    cmap.one_link(DartId(15), DartId(16));
    cmap.one_link(DartId(16), DartId(13));

    cmap.one_link(DartId(17), DartId(18));
    cmap.one_link(DartId(18), DartId(19));
    cmap.one_link(DartId(19), DartId(20));
    cmap.one_link(DartId(20), DartId(21));
    cmap.one_link(DartId(21), DartId(22));
    cmap.one_link(DartId(22), DartId(23));
    cmap.one_link(DartId(23), DartId(24));
    cmap.one_link(DartId(24), DartId(25));
    cmap.one_link(DartId(25), DartId(17));

    cmap.one_link(DartId(26), DartId(27));
    cmap.one_link(DartId(27), DartId(28));
    cmap.one_link(DartId(28), DartId(26));

    cmap.two_link(DartId(3), DartId(7));
    cmap.two_link(DartId(4), DartId(13));
    cmap.two_link(DartId(10), DartId(27));
    cmap.two_link(DartId(11), DartId(26));
    cmap.two_link(DartId(12), DartId(14));
    cmap.two_link(DartId(15), DartId(17));
    cmap.two_link(DartId(18), DartId(28));

    // geometry
    cmap.insert_vertex(VertexId(1), (1.0, 0.0));
    cmap.insert_vertex(VertexId(2), (2.0, 0.0));
    cmap.insert_vertex(VertexId(3), (2.5, 0.5));
    cmap.insert_vertex(VertexId(4), (2.0, 1.0));
    cmap.insert_vertex(VertexId(5), (1.0, 1.0));
    cmap.insert_vertex(VertexId(6), (0.5, 0.5));
    cmap.insert_vertex(VertexId(9), (3.0, 1.0));
    cmap.insert_vertex(VertexId(10), (3.0, 2.0));
    cmap.insert_vertex(VertexId(11), (2.5, 1.0));
    cmap.insert_vertex(VertexId(12), (2.0, 2.0));
    cmap.insert_vertex(VertexId(16), (1.0, 2.0));
    cmap.insert_vertex(VertexId(20), (3.0, 3.0));
    cmap.insert_vertex(VertexId(21), (2.7, 3.0));
    cmap.insert_vertex(VertexId(22), (2.7, 2.3));
    cmap.insert_vertex(VertexId(23), (1.3, 2.3));
    cmap.insert_vertex(VertexId(24), (1.3, 3.0));
    cmap.insert_vertex(VertexId(25), (1.0, 3.0));

    cmap
}

#[test]
fn fan_cells() {
    // generate a map with all kinds of cell
    let mut map = generate_map();
    // we know these by construction
    let hex1 = FaceId(1);
    let hex2 = FaceId(7);
    let squ = FaceId(13);
    let nop = FaceId(17);
    let tri = FaceId(26);

    // the hex will be
    let nd = map.add_free_darts(6).0;
    let new_darts = (nd..nd + 6).map(DartId).collect::<Vec<_>>();
    assert!(fan_cell(&mut map, hex1, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex1.into()).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(3)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(4)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(5)).count(), 3);

    // the hex will be
    let nd = map.add_free_darts(6).0;
    let new_darts = (nd..nd + 6).map(DartId).collect::<Vec<_>>();
    assert!(fan_cell(&mut map, hex2, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex2.into()).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(8)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(10)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(11)).count(), 3);

    // the square will be split in two
    let nd = map.add_free_darts(2).0;
    let new_darts = (nd..nd + 2).map(DartId).collect::<Vec<_>>();
    assert!(fan_cell(&mut map, squ, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(squ.into()).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(15)).count(), 3);

    // this will be a no-op since the polygon isn't fannable
    let nd = map.add_free_darts(12).0;
    let new_darts = (nd..nd + 12).map(DartId).collect::<Vec<_>>();
    assert_eq!(
        fan_cell(&mut map, nop, &new_darts),
        Err(TriangulateError::NonFannable)
    );

    assert_eq!(map.i_cell::<2>(nop.into()).count(), 9); // unchanged

    assert_eq!(
        fan_cell(&mut map, tri, &[]),
        Err(TriangulateError::AlreadyTriangulated)
    );

    assert_eq!(map.i_cell::<2>(tri.into()).count(), 3); // unchanged
}

#[test]
fn earclip_cells() {
    // generate a map with all kinds of cell
    let mut map = generate_map();
    // we know these by construction
    let hex1 = FaceId(1);
    let hex2 = FaceId(7);
    let squ = FaceId(13);
    let smh = FaceId(17);
    let tri = FaceId(26);

    // the hex will be split in 4
    let nd = map.add_free_darts(6).0;
    let new_darts = (nd..nd + 6).map(DartId).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, hex1, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex1.into()).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(3)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(4)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(5)).count(), 3);

    // the hex will be split in 4
    let nd = map.add_free_darts(6).0;
    let new_darts = (nd..nd + 6).map(DartId).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, hex2, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(hex2.into()).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(8)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(10)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(11)).count(), 3);

    // the square will be split in 2
    let nd = map.add_free_darts(2).0;
    let new_darts = (nd..nd + 2).map(DartId).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, squ, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(squ.into()).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(15)).count(), 3);

    // 9-gon is split in 7
    let nd = map.add_free_darts(12).0;
    let new_darts = (nd..nd + 12).map(DartId).collect::<Vec<_>>();
    assert!(earclip_cell(&mut map, smh, &new_darts).is_ok());

    assert_eq!(map.i_cell::<2>(smh.into()).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(18)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(19)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(21)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(22)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(23)).count(), 3);
    assert_eq!(map.i_cell::<2>(DartId(24)).count(), 3);

    assert_eq!(
        earclip_cell(&mut map, tri, &[]),
        Err(TriangulateError::AlreadyTriangulated)
    );

    assert_eq!(map.i_cell::<2>(tri.into()).count(), 3); // unchanged
}
