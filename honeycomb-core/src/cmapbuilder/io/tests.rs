// ------ IMPORTS

use crate::{CMap2, DartIdentifier, Orbit2, OrbitPolicy};
use vtkio::Vtk;

// ------ CONTENT

#[cfg(feature = "io")]
#[test]
fn io_write() {
    // build a map looking like this:
    //      15
    //     / \
    //    /   \
    //   /     \
    //  16      14
    //  |       |
    //  4---3---7
    //  |   |  /|
    //  |   | / |
    //  |   |/  |
    //  1---2---6
    let mut cmap: CMap2<f32> = CMap2::new(16);
    // bottom left square
    cmap.one_link(1, 2);
    cmap.one_link(2, 3);
    cmap.one_link(3, 4);
    cmap.one_link(4, 1);
    // bottom right triangles
    cmap.one_link(5, 6);
    cmap.one_link(6, 7);
    cmap.one_link(7, 5);
    cmap.two_link(7, 8);
    cmap.one_link(8, 9);
    cmap.one_link(9, 10);
    cmap.one_link(10, 8);
    // top polygon
    cmap.one_link(11, 12);
    cmap.one_link(12, 13);
    cmap.one_link(13, 14);
    cmap.one_link(14, 15);
    cmap.one_link(15, 16);
    cmap.one_link(16, 11);
    // assemble
    cmap.two_link(2, 10);
    cmap.two_link(3, 11);
    cmap.two_link(9, 12);

    // insert vertices
    cmap.insert_vertex(1, (0.0, 0.0));
    cmap.insert_vertex(2, (1.0, 0.0));
    cmap.insert_vertex(6, (2.0, 0.0));
    cmap.insert_vertex(4, (0.0, 1.0));
    cmap.insert_vertex(3, (1.0, 1.0));
    cmap.insert_vertex(7, (2.0, 1.0));
    cmap.insert_vertex(16, (0.0, 2.0));
    cmap.insert_vertex(15, (1.0, 3.0));
    cmap.insert_vertex(14, (2.0, 2.0));

    // generate VTK data
    let mut res = String::new();
    cmap.to_vtk_ascii(&mut res);
    println!("{res}");

    // check result
    assert!(res.contains("POINTS 9 float"));
    assert!(res.contains("CELLS 12 44"));
    assert!(res.contains("CELL_TYPES 12"));
    // faces
    assert!(res.contains("4 0 1 2 3"));
    assert!(res.contains("3 1 4 5"));
    assert!(res.contains("4 0 1 2 3"));
    assert!(res.contains("4 0 1 2 3"));
    // edges
    assert!(res.contains("2 0 1"));
    assert!(res.contains("2 3 0"));
    assert!(res.contains("2 1 4"));
    assert!(res.contains("2 4 5"));
    assert!(res.contains("2 5 6"));
    assert!(res.contains("2 6 7"));
    assert!(res.contains("2 7 8"));
    assert!(res.contains("2 8 3"));
}

#[test]
fn io_read() {
    let vtk = Vtk::parse_legacy_be(VTK_ASCII).unwrap();

    let cmap: CMap2<f32> = super::building_routines::build2_from_vtk(vtk);

    // check result
    let faces = cmap.fetch_faces();
    assert_eq!(faces.identifiers.len(), 4);
    assert_eq!(cmap.fetch_edges().identifiers.len(), 12);
    assert_eq!(cmap.fetch_vertices().identifiers.len(), 9);

    let mut n_vertices_per_face: Vec<usize> = faces
        .identifiers
        .iter()
        .map(|id| Orbit2::new(&cmap, OrbitPolicy::Face, *id as DartIdentifier).count())
        .collect();
    let (mut three_count, mut four_count, mut six_count): (usize, usize, usize) = (0, 0, 0);
    while let Some(n) = n_vertices_per_face.pop() {
        match n {
            3 => three_count += 1,
            4 => four_count += 1,
            6 => six_count += 1,
            _ => panic!("cmap was built incorrectly"),
        }
    }
    assert_eq!(three_count, 2);
    assert_eq!(four_count, 1);
    assert_eq!(six_count, 1);
}

#[cfg(all(test, feature = "io"))]
const VTK_ASCII: &[u8] = b"
# vtk DataFile Version 2.0
cmap
ASCII

DATASET UNSTRUCTURED_GRID
POINTS 9 float
0 0 0  1 0 0  1 1 0
0 1 0  2 0 0  2 1 0
2 2 0  1 3 0  0 2 0

CELLS 17 54
1 0
1 4
1 6
1 7
1 8
2 0 1
2 3 0
2 1 4
2 4 5
2 5 6
2 6 7
2 7 8
2 8 3
4 0 1 2 3
3 1 4 5
3 1 5 2
6 3 2 5 6 7 8

CELL_TYPES 17
1
1
1
1
1
3
3
3
3
3
3
3
3
9
5
5
7


POINT_DATA 9

CELL_DATA 17
";
