// ------ IMPORTS

use crate::{CMap2, DartIdentifier, Orbit2, OrbitPolicy};
use vtkio::Vtk;

// ------ CONTENT

#[test]
fn io_read() {
    let vtk = Vtk::parse_legacy_be(VTK_ASCII).unwrap();

    let cmap: CMap2<f32> = super::build_2d_from_vtk(vtk);

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
