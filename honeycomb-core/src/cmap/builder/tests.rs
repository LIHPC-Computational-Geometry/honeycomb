use crate::attributes::AttrStorageManager;
use crate::prelude::{CMap2, CMapBuilder, DartIdType, GridDescriptor, Orbit2, OrbitPolicy};

use vtkio::Vtk;

// --- basic

#[test]
fn example_test() {
    let builder = CMapBuilder::default().n_darts(10);
    let cmap: CMap2<f64> = builder.build().unwrap();
    assert_eq!(cmap.n_darts(), 11);
}

// --- grid

#[test]
fn build_nc_lpc_l() {
    let descriptor = GridDescriptor::default()
        .n_cells([4, 4, 0])
        .len_per_cell([1.0_f64, 1.0_f64, 1.0_f64])
        .lens([4.0_f64, 4.0_f64, 4.0_f64]);
    assert!(descriptor.clone().parse_2d().is_ok());
    assert!(descriptor.split_quads(true).parse_2d().is_ok());
}

#[test]
fn build_nc_lpc() {
    let descriptor = GridDescriptor::default()
        .n_cells([4, 4, 0])
        .len_per_cell([1.0_f64, 1.0_f64, 1.0_f64]);
    assert!(descriptor.clone().parse_2d().is_ok());
    assert!(descriptor.split_quads(true).parse_2d().is_ok());
}

#[test]
fn build_nc_l() {
    let descriptor = GridDescriptor::default()
        .n_cells_x(4)
        .n_cells_y(4)
        .lens([4.0_f64, 4.0_f64, 4.0_f64]);
    assert!(descriptor.clone().parse_2d().is_ok());
    assert!(descriptor.split_quads(true).parse_2d().is_ok());
}

#[test]
fn build_lpc_l() {
    let descriptor = GridDescriptor::default()
        .len_per_cell_x(1.0_f64)
        .len_per_cell_y(1.0_f64)
        .lens_x(4.0)
        .lens_y(4.0);
    assert!(descriptor.clone().parse_2d().is_ok());
    assert!(descriptor.split_quads(true).parse_2d().is_ok());
}

#[test]
fn build_incomplete() {
    assert!(GridDescriptor::default()
        .len_per_cell([1.0_f64, 1.0_f64, 1.0_f64])
        .parse_2d()
        .is_err());
    assert!(<GridDescriptor<f64>>::default()
        .n_cells([4, 4, 0])
        .parse_2d()
        .is_err());
    assert!(GridDescriptor::default()
        .lens([4.0_f64, 4.0_f64, 4.0_f64])
        .parse_2d()
        .is_err());
}

#[test]
#[should_panic(expected = "length per y cell is null or negative")]
fn build_neg_lpc() {
    let tmp = GridDescriptor::default()
        .n_cells([4, 4, 0])
        .len_per_cell([1.0_f64, -1.0_f64, 1.0_f64])
        .parse_2d();
    let _ = tmp.unwrap(); // panic on Err(BuilderError::InvalidParameters)
}

#[test]
#[should_panic(expected = "grid length along x is null or negative")]
fn build_null_l() {
    let tmp = GridDescriptor::default()
        .n_cells([4, 4, 0])
        .lens([0.0_f64, 4.0_f64, 4.0_f64])
        .parse_2d();
    let _ = tmp.unwrap(); // panic on Err(BuilderError::InvalidParameters)
}

#[test]
#[should_panic(expected = "length per x cell is null or negative")]
fn build_neg_lpc_neg_l() {
    // lpc are parsed first so their panic msg should be the one to pop
    // x val is parsed first so ...
    let tmp = GridDescriptor::default()
        .len_per_cell([-1.0_f64, -1.0_f64, 1.0_f64])
        .lens([0.0_f64, 4.0_f64, 4.0_f64])
        .parse_2d();
    let _ = tmp.unwrap(); // panic on Err(BuilderError::InvalidParameters)
}

// --- grid building

#[test]
fn square_cmap2_correctness() {
    let descriptor = GridDescriptor::default()
        .n_cells([2, 2, 2])
        .len_per_cell([1., 1., 1.]);
    let cmap: CMap2<f64> = CMapBuilder::from(descriptor).build().unwrap();

    // hardcoded because using a generic loop & dim would just mean
    // reusing the same pattern as the one used during construction

    // face 0
    assert_eq!(cmap.face_id(1), 1);
    assert_eq!(cmap.face_id(2), 1);
    assert_eq!(cmap.face_id(3), 1);
    assert_eq!(cmap.face_id(4), 1);

    // i-cell uses beta 0 to ensure correctness, so the iterator is BFS-like
    let mut face = cmap.i_cell::<2>(1);
    assert_eq!(face.next(), Some(1));
    assert_eq!(face.next(), Some(2)); // b1
    assert_eq!(face.next(), Some(4)); // b0
    assert_eq!(face.next(), Some(3)); // b1b1
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(1), 2);
    assert_eq!(cmap.beta::<1>(2), 3);
    assert_eq!(cmap.beta::<1>(3), 4);
    assert_eq!(cmap.beta::<1>(4), 1);

    assert_eq!(cmap.beta::<2>(1), 0);
    assert_eq!(cmap.beta::<2>(2), 8);
    assert_eq!(cmap.beta::<2>(3), 9);
    assert_eq!(cmap.beta::<2>(4), 0);

    // face 1
    assert_eq!(cmap.face_id(5), 5);
    assert_eq!(cmap.face_id(6), 5);
    assert_eq!(cmap.face_id(7), 5);
    assert_eq!(cmap.face_id(8), 5);

    let mut face = cmap.i_cell::<2>(5);
    assert_eq!(face.next(), Some(5));
    assert_eq!(face.next(), Some(6));
    assert_eq!(face.next(), Some(8));
    assert_eq!(face.next(), Some(7));
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(5), 6);
    assert_eq!(cmap.beta::<1>(6), 7);
    assert_eq!(cmap.beta::<1>(7), 8);
    assert_eq!(cmap.beta::<1>(8), 5);

    assert_eq!(cmap.beta::<2>(5), 0);
    assert_eq!(cmap.beta::<2>(6), 0);
    assert_eq!(cmap.beta::<2>(7), 13);
    assert_eq!(cmap.beta::<2>(8), 2);

    // face 2
    assert_eq!(cmap.face_id(9), 9);
    assert_eq!(cmap.face_id(10), 9);
    assert_eq!(cmap.face_id(11), 9);
    assert_eq!(cmap.face_id(12), 9);

    let mut face = cmap.i_cell::<2>(9);
    assert_eq!(face.next(), Some(9));
    assert_eq!(face.next(), Some(10));
    assert_eq!(face.next(), Some(12));
    assert_eq!(face.next(), Some(11));
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(9), 10);
    assert_eq!(cmap.beta::<1>(10), 11);
    assert_eq!(cmap.beta::<1>(11), 12);
    assert_eq!(cmap.beta::<1>(12), 9);

    assert_eq!(cmap.beta::<2>(9), 3);
    assert_eq!(cmap.beta::<2>(10), 16);
    assert_eq!(cmap.beta::<2>(11), 0);
    assert_eq!(cmap.beta::<2>(12), 0);

    // face 3
    assert_eq!(cmap.face_id(13), 13);
    assert_eq!(cmap.face_id(14), 13);
    assert_eq!(cmap.face_id(15), 13);
    assert_eq!(cmap.face_id(16), 13);

    let mut face = cmap.i_cell::<2>(13);
    assert_eq!(face.next(), Some(13));
    assert_eq!(face.next(), Some(14));
    assert_eq!(face.next(), Some(16));
    assert_eq!(face.next(), Some(15));
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(13), 14);
    assert_eq!(cmap.beta::<1>(14), 15);
    assert_eq!(cmap.beta::<1>(15), 16);
    assert_eq!(cmap.beta::<1>(16), 13);

    assert_eq!(cmap.beta::<2>(13), 7);
    assert_eq!(cmap.beta::<2>(14), 0);
    assert_eq!(cmap.beta::<2>(15), 0);
    assert_eq!(cmap.beta::<2>(16), 10);
}

#[allow(clippy::too_many_lines)]
#[test]
fn splitsquare_cmap2_correctness() {
    let cmap: CMap2<f64> = CMapBuilder::unit_triangles(2).build().unwrap();

    // hardcoded because using a generic loop & dim would just mean
    // reusing the same pattern as the one used during construction

    // face 1
    assert_eq!(cmap.face_id(1), 1);
    assert_eq!(cmap.face_id(2), 1);
    assert_eq!(cmap.face_id(3), 1);

    let mut face = cmap.i_cell::<2>(1);
    assert_eq!(face.next(), Some(1));
    assert_eq!(face.next(), Some(2));
    assert_eq!(face.next(), Some(3));

    assert_eq!(cmap.beta::<1>(1), 2);
    assert_eq!(cmap.beta::<1>(2), 3);
    assert_eq!(cmap.beta::<1>(3), 1);

    assert_eq!(cmap.beta::<2>(1), 0);
    assert_eq!(cmap.beta::<2>(2), 4);
    assert_eq!(cmap.beta::<2>(3), 0);

    // face 4
    assert_eq!(cmap.face_id(4), 4);
    assert_eq!(cmap.face_id(5), 4);
    assert_eq!(cmap.face_id(6), 4);

    let mut face = cmap.i_cell::<2>(4);
    assert_eq!(face.next(), Some(4));
    assert_eq!(face.next(), Some(5));
    assert_eq!(face.next(), Some(6));

    assert_eq!(cmap.beta::<1>(4), 5);
    assert_eq!(cmap.beta::<1>(5), 6);
    assert_eq!(cmap.beta::<1>(6), 4);

    assert_eq!(cmap.beta::<2>(4), 2);
    assert_eq!(cmap.beta::<2>(5), 9);
    assert_eq!(cmap.beta::<2>(6), 13);

    // face 7
    assert_eq!(cmap.face_id(7), 7);
    assert_eq!(cmap.face_id(8), 7);
    assert_eq!(cmap.face_id(9), 7);

    let mut face = cmap.i_cell::<2>(7);
    assert_eq!(face.next(), Some(7));
    assert_eq!(face.next(), Some(8));
    assert_eq!(face.next(), Some(9));

    assert_eq!(cmap.beta::<1>(7), 8);
    assert_eq!(cmap.beta::<1>(8), 9);
    assert_eq!(cmap.beta::<1>(9), 7);

    assert_eq!(cmap.beta::<2>(7), 0);
    assert_eq!(cmap.beta::<2>(8), 10);
    assert_eq!(cmap.beta::<2>(9), 5);

    // face 10
    assert_eq!(cmap.face_id(10), 10);
    assert_eq!(cmap.face_id(11), 10);
    assert_eq!(cmap.face_id(12), 10);

    let mut face = cmap.i_cell::<2>(10);
    assert_eq!(face.next(), Some(10));
    assert_eq!(face.next(), Some(11));
    assert_eq!(face.next(), Some(12));

    assert_eq!(cmap.beta::<1>(10), 11);
    assert_eq!(cmap.beta::<1>(11), 12);
    assert_eq!(cmap.beta::<1>(12), 10);

    assert_eq!(cmap.beta::<2>(10), 8);
    assert_eq!(cmap.beta::<2>(11), 0);
    assert_eq!(cmap.beta::<2>(12), 19);

    // face 13
    assert_eq!(cmap.face_id(13), 13);
    assert_eq!(cmap.face_id(14), 13);
    assert_eq!(cmap.face_id(15), 13);

    let mut face = cmap.i_cell::<2>(13);
    assert_eq!(face.next(), Some(13));
    assert_eq!(face.next(), Some(14));
    assert_eq!(face.next(), Some(15));

    assert_eq!(cmap.beta::<1>(13), 14);
    assert_eq!(cmap.beta::<1>(14), 15);
    assert_eq!(cmap.beta::<1>(15), 13);

    assert_eq!(cmap.beta::<2>(13), 6);
    assert_eq!(cmap.beta::<2>(14), 16);
    assert_eq!(cmap.beta::<2>(15), 0);

    // face 16
    assert_eq!(cmap.face_id(16), 16);
    assert_eq!(cmap.face_id(17), 16);
    assert_eq!(cmap.face_id(18), 16);

    let mut face = cmap.i_cell::<2>(16);
    assert_eq!(face.next(), Some(16));
    assert_eq!(face.next(), Some(17));
    assert_eq!(face.next(), Some(18));

    assert_eq!(cmap.beta::<1>(16), 17);
    assert_eq!(cmap.beta::<1>(17), 18);
    assert_eq!(cmap.beta::<1>(18), 16);

    assert_eq!(cmap.beta::<2>(16), 14);
    assert_eq!(cmap.beta::<2>(17), 21);
    assert_eq!(cmap.beta::<2>(18), 0);

    // face 19
    assert_eq!(cmap.face_id(19), 19);
    assert_eq!(cmap.face_id(20), 19);
    assert_eq!(cmap.face_id(21), 19);

    let mut face = cmap.i_cell::<2>(19);
    assert_eq!(face.next(), Some(19));
    assert_eq!(face.next(), Some(20));
    assert_eq!(face.next(), Some(21));

    assert_eq!(cmap.beta::<1>(19), 20);
    assert_eq!(cmap.beta::<1>(20), 21);
    assert_eq!(cmap.beta::<1>(21), 19);

    assert_eq!(cmap.beta::<2>(19), 12);
    assert_eq!(cmap.beta::<2>(20), 22);
    assert_eq!(cmap.beta::<2>(21), 17);

    // face 22
    assert_eq!(cmap.face_id(22), 22);
    assert_eq!(cmap.face_id(23), 22);
    assert_eq!(cmap.face_id(24), 22);

    let mut face = cmap.i_cell::<2>(22);
    assert_eq!(face.next(), Some(22));
    assert_eq!(face.next(), Some(23));
    assert_eq!(face.next(), Some(24));

    assert_eq!(cmap.beta::<1>(22), 23);
    assert_eq!(cmap.beta::<1>(23), 24);
    assert_eq!(cmap.beta::<1>(24), 22);

    assert_eq!(cmap.beta::<2>(22), 20);
    assert_eq!(cmap.beta::<2>(23), 0);
    assert_eq!(cmap.beta::<2>(24), 0);
}

// --- IO

#[test]
fn io_read() {
    let vtk = Vtk::parse_legacy_be(VTK_ASCII).unwrap();
    // unwrap is fine since we know the VTK_ASCII const is correct
    let cmap: CMap2<f32> =
        super::io::build_2d_from_vtk(vtk, AttrStorageManager::default()).unwrap();

    // check result
    let faces: Vec<_> = cmap.iter_faces().collect();
    assert_eq!(faces.len(), 4);
    assert_eq!(cmap.iter_edges().count(), 12);
    assert_eq!(cmap.iter_vertices().count(), 9);

    let mut n_vertices_per_face: Vec<usize> = faces
        .iter()
        .map(|id| Orbit2::new(&cmap, OrbitPolicy::Face, *id as DartIdType).count())
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
