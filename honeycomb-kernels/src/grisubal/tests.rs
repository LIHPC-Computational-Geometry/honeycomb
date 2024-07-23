// ------ IMPORTS

use honeycomb_core::Vertex2;
use vtkio::Vtk;

use crate::Geometry2;

// ------ CONTENT

// --- geometry building

#[cfg(test)]
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

#[test]
fn build_valid_geometry() {
    // parse vtk
    let vtk = Vtk::parse_legacy_be(VTK_ASCII).unwrap();
    // build geometry
    let geometry: Geometry2<f32> = Geometry2::from(vtk);
    // check results; we're expecting:
    // - 9 vertices
    // - 8 segments making up the boundaries
    // - 5 points of interest out of the 9 vertices
    let Geometry2 {
        vertices,
        segments,
        poi,
    } = geometry;
    // vertices
    assert_eq!(vertices.len(), 9);
    assert!(vertices.contains(&Vertex2::from((0., 0.))));
    assert!(vertices.contains(&Vertex2::from((1., 0.))));
    assert!(vertices.contains(&Vertex2::from((2., 0.))));
    assert!(vertices.contains(&Vertex2::from((0., 1.))));
    assert!(vertices.contains(&Vertex2::from((1., 1.))));
    assert!(vertices.contains(&Vertex2::from((2., 1.))));
    assert!(vertices.contains(&Vertex2::from((0., 2.))));
    assert!(vertices.contains(&Vertex2::from((1., 3.))));
    assert!(vertices.contains(&Vertex2::from((2., 2.))));
    // segments
    assert_eq!(segments.len(), 8);
    assert!(segments.contains(&(0, 1)));
    assert!(segments.contains(&(3, 0)));
    assert!(segments.contains(&(1, 4)));
    assert!(segments.contains(&(4, 5)));
    assert!(segments.contains(&(5, 6)));
    assert!(segments.contains(&(6, 7)));
    assert!(segments.contains(&(7, 8)));
    assert!(segments.contains(&(8, 3)));
    // poi
    assert_eq!(poi.len(), 5);
    assert!(poi.contains(&0));
    assert!(poi.contains(&4));
    assert!(poi.contains(&6));
    assert!(poi.contains(&7));
    assert!(poi.contains(&8));
}
