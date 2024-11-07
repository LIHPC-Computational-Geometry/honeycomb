// ------ IMPORTS

use crate::cmap::{CMap2, CMapBuilder, DartId, FaceId, GridDescriptor};

// ------ CONTENT

// --- descriptor

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
    assert_eq!(cmap.face_id(DartId(1)), FaceId(1));
    assert_eq!(cmap.face_id(DartId(2)), FaceId(1));
    assert_eq!(cmap.face_id(DartId(3)), FaceId(1));
    assert_eq!(cmap.face_id(DartId(4)), FaceId(1));

    let mut face = cmap.i_cell::<2>(DartId(1));
    assert_eq!(face.next(), Some(DartId(1)));
    assert_eq!(face.next(), Some(DartId(2)));
    assert_eq!(face.next(), Some(DartId(3)));
    assert_eq!(face.next(), Some(DartId(4)));
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(DartId(1)), DartId(2));
    assert_eq!(cmap.beta::<1>(DartId(2)), DartId(3));
    assert_eq!(cmap.beta::<1>(DartId(3)), DartId(4));
    assert_eq!(cmap.beta::<1>(DartId(4)), DartId(1));

    assert_eq!(cmap.beta::<2>(DartId(1)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(2)), DartId(8));
    assert_eq!(cmap.beta::<2>(DartId(3)), DartId(9));
    assert_eq!(cmap.beta::<2>(DartId(4)), DartId(0));

    // face 1
    assert_eq!(cmap.face_id(DartId(5)), FaceId(5));
    assert_eq!(cmap.face_id(DartId(6)), FaceId(5));
    assert_eq!(cmap.face_id(DartId(7)), FaceId(5));
    assert_eq!(cmap.face_id(DartId(8)), FaceId(5));

    let mut face = cmap.i_cell::<2>(DartId(5));
    assert_eq!(face.next(), Some(DartId(5)));
    assert_eq!(face.next(), Some(DartId(6)));
    assert_eq!(face.next(), Some(DartId(7)));
    assert_eq!(face.next(), Some(DartId(8)));
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(DartId(5)), DartId(6));
    assert_eq!(cmap.beta::<1>(DartId(6)), DartId(7));
    assert_eq!(cmap.beta::<1>(DartId(7)), DartId(8));
    assert_eq!(cmap.beta::<1>(DartId(8)), DartId(5));

    assert_eq!(cmap.beta::<2>(DartId(5)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(6)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(7)), DartId(13));
    assert_eq!(cmap.beta::<2>(DartId(8)), DartId(2));

    // face 2
    assert_eq!(cmap.face_id(DartId(9)), FaceId(9));
    assert_eq!(cmap.face_id(DartId(10)), FaceId(9));
    assert_eq!(cmap.face_id(DartId(11)), FaceId(9));
    assert_eq!(cmap.face_id(DartId(12)), FaceId(9));

    let mut face = cmap.i_cell::<2>(DartId(9));
    assert_eq!(face.next(), Some(DartId(9)));
    assert_eq!(face.next(), Some(DartId(10)));
    assert_eq!(face.next(), Some(DartId(11)));
    assert_eq!(face.next(), Some(DartId(12)));
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(DartId(9)), DartId(10));
    assert_eq!(cmap.beta::<1>(DartId(10)), DartId(11));
    assert_eq!(cmap.beta::<1>(DartId(11)), DartId(12));
    assert_eq!(cmap.beta::<1>(DartId(12)), DartId(9));

    assert_eq!(cmap.beta::<2>(DartId(9)), DartId(3));
    assert_eq!(cmap.beta::<2>(DartId(10)), DartId(16));
    assert_eq!(cmap.beta::<2>(DartId(11)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(12)), DartId(0));

    // face 3
    assert_eq!(cmap.face_id(DartId(13)), FaceId(13));
    assert_eq!(cmap.face_id(DartId(14)), FaceId(13));
    assert_eq!(cmap.face_id(DartId(15)), FaceId(13));
    assert_eq!(cmap.face_id(DartId(16)), FaceId(13));

    let mut face = cmap.i_cell::<2>(DartId(13));
    assert_eq!(face.next(), Some(DartId(13)));
    assert_eq!(face.next(), Some(DartId(14)));
    assert_eq!(face.next(), Some(DartId(15)));
    assert_eq!(face.next(), Some(DartId(16)));
    assert_eq!(face.next(), None);

    assert_eq!(cmap.beta::<1>(DartId(13)), DartId(14));
    assert_eq!(cmap.beta::<1>(DartId(14)), DartId(15));
    assert_eq!(cmap.beta::<1>(DartId(15)), DartId(16));
    assert_eq!(cmap.beta::<1>(DartId(16)), DartId(13));

    assert_eq!(cmap.beta::<2>(DartId(13)), DartId(7));
    assert_eq!(cmap.beta::<2>(DartId(14)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(15)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(16)), DartId(10));
}

#[allow(clippy::too_many_lines)]
#[test]
fn splitsquare_cmap2_correctness() {
    let cmap: CMap2<f64> = CMapBuilder::unit_triangles(2).build().unwrap();

    // hardcoded because using a generic loop & dim would just mean
    // reusing the same pattern as the one used during construction

    // face 1
    assert_eq!(cmap.face_id(DartId(1)), FaceId(1));
    assert_eq!(cmap.face_id(DartId(2)), FaceId(1));
    assert_eq!(cmap.face_id(DartId(3)), FaceId(1));

    let mut face = cmap.i_cell::<2>(DartId(1));
    assert_eq!(face.next(), Some(DartId(1)));
    assert_eq!(face.next(), Some(DartId(2)));
    assert_eq!(face.next(), Some(DartId(3)));

    assert_eq!(cmap.beta::<1>(DartId(1)), DartId(2));
    assert_eq!(cmap.beta::<1>(DartId(2)), DartId(3));
    assert_eq!(cmap.beta::<1>(DartId(3)), DartId(1));

    assert_eq!(cmap.beta::<2>(DartId(1)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(2)), DartId(4));
    assert_eq!(cmap.beta::<2>(DartId(3)), DartId(0));

    // face 4
    assert_eq!(cmap.face_id(DartId(4)), FaceId(4));
    assert_eq!(cmap.face_id(DartId(5)), FaceId(4));
    assert_eq!(cmap.face_id(DartId(6)), FaceId(4));

    let mut face = cmap.i_cell::<2>(DartId(4));
    assert_eq!(face.next(), Some(DartId(4)));
    assert_eq!(face.next(), Some(DartId(5)));
    assert_eq!(face.next(), Some(DartId(6)));

    assert_eq!(cmap.beta::<1>(DartId(4)), DartId(5));
    assert_eq!(cmap.beta::<1>(DartId(5)), DartId(6));
    assert_eq!(cmap.beta::<1>(DartId(6)), DartId(4));

    assert_eq!(cmap.beta::<2>(DartId(4)), DartId(2));
    assert_eq!(cmap.beta::<2>(DartId(5)), DartId(9));
    assert_eq!(cmap.beta::<2>(DartId(6)), DartId(13));

    // face 7
    assert_eq!(cmap.face_id(DartId(7)), FaceId(7));
    assert_eq!(cmap.face_id(DartId(8)), FaceId(7));
    assert_eq!(cmap.face_id(DartId(9)), FaceId(7));

    let mut face = cmap.i_cell::<2>(DartId(7));
    assert_eq!(face.next(), Some(DartId(7)));
    assert_eq!(face.next(), Some(DartId(8)));
    assert_eq!(face.next(), Some(DartId(9)));

    assert_eq!(cmap.beta::<1>(DartId(7)), DartId(8));
    assert_eq!(cmap.beta::<1>(DartId(8)), DartId(9));
    assert_eq!(cmap.beta::<1>(DartId(9)), DartId(7));

    assert_eq!(cmap.beta::<2>(DartId(7)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(8)), DartId(10));
    assert_eq!(cmap.beta::<2>(DartId(9)), DartId(5));

    // face 10
    assert_eq!(cmap.face_id(DartId(10)), FaceId(10));
    assert_eq!(cmap.face_id(DartId(11)), FaceId(10));
    assert_eq!(cmap.face_id(DartId(12)), FaceId(10));

    let mut face = cmap.i_cell::<2>(DartId(10));
    assert_eq!(face.next(), Some(DartId(10)));
    assert_eq!(face.next(), Some(DartId(11)));
    assert_eq!(face.next(), Some(DartId(12)));

    assert_eq!(cmap.beta::<1>(DartId(10)), DartId(11));
    assert_eq!(cmap.beta::<1>(DartId(11)), DartId(12));
    assert_eq!(cmap.beta::<1>(DartId(12)), DartId(10));

    assert_eq!(cmap.beta::<2>(DartId(10)), DartId(8));
    assert_eq!(cmap.beta::<2>(DartId(11)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(12)), DartId(19));

    // face 13
    assert_eq!(cmap.face_id(DartId(13)), FaceId(13));
    assert_eq!(cmap.face_id(DartId(14)), FaceId(13));
    assert_eq!(cmap.face_id(DartId(15)), FaceId(13));

    let mut face = cmap.i_cell::<2>(DartId(13));
    assert_eq!(face.next(), Some(DartId(13)));
    assert_eq!(face.next(), Some(DartId(14)));
    assert_eq!(face.next(), Some(DartId(15)));

    assert_eq!(cmap.beta::<1>(DartId(13)), DartId(14));
    assert_eq!(cmap.beta::<1>(DartId(14)), DartId(15));
    assert_eq!(cmap.beta::<1>(DartId(15)), DartId(13));

    assert_eq!(cmap.beta::<2>(DartId(13)), DartId(6));
    assert_eq!(cmap.beta::<2>(DartId(14)), DartId(16));
    assert_eq!(cmap.beta::<2>(DartId(15)), DartId(0));

    // face 16
    assert_eq!(cmap.face_id(DartId(16)), FaceId(16));
    assert_eq!(cmap.face_id(DartId(17)), FaceId(16));
    assert_eq!(cmap.face_id(DartId(18)), FaceId(16));

    let mut face = cmap.i_cell::<2>(DartId(16));
    assert_eq!(face.next(), Some(DartId(16)));
    assert_eq!(face.next(), Some(DartId(17)));
    assert_eq!(face.next(), Some(DartId(18)));

    assert_eq!(cmap.beta::<1>(DartId(16)), DartId(17));
    assert_eq!(cmap.beta::<1>(DartId(17)), DartId(18));
    assert_eq!(cmap.beta::<1>(DartId(18)), DartId(16));

    assert_eq!(cmap.beta::<2>(DartId(16)), DartId(14));
    assert_eq!(cmap.beta::<2>(DartId(17)), DartId(21));
    assert_eq!(cmap.beta::<2>(DartId(18)), DartId(0));

    // face 19
    assert_eq!(cmap.face_id(DartId(19)), FaceId(19));
    assert_eq!(cmap.face_id(DartId(20)), FaceId(19));
    assert_eq!(cmap.face_id(DartId(21)), FaceId(19));

    let mut face = cmap.i_cell::<2>(DartId(19));
    assert_eq!(face.next(), Some(DartId(19)));
    assert_eq!(face.next(), Some(DartId(20)));
    assert_eq!(face.next(), Some(DartId(21)));

    assert_eq!(cmap.beta::<1>(DartId(19)), DartId(20));
    assert_eq!(cmap.beta::<1>(DartId(20)), DartId(21));
    assert_eq!(cmap.beta::<1>(DartId(21)), DartId(19));

    assert_eq!(cmap.beta::<2>(DartId(19)), DartId(12));
    assert_eq!(cmap.beta::<2>(DartId(20)), DartId(22));
    assert_eq!(cmap.beta::<2>(DartId(21)), DartId(17));

    // face 22
    assert_eq!(cmap.face_id(DartId(22)), FaceId(22));
    assert_eq!(cmap.face_id(DartId(23)), FaceId(22));
    assert_eq!(cmap.face_id(DartId(24)), FaceId(22));

    let mut face = cmap.i_cell::<2>(DartId(22));
    assert_eq!(face.next(), Some(DartId(22)));
    assert_eq!(face.next(), Some(DartId(23)));
    assert_eq!(face.next(), Some(DartId(24)));

    assert_eq!(cmap.beta::<1>(DartId(22)), DartId(23));
    assert_eq!(cmap.beta::<1>(DartId(23)), DartId(24));
    assert_eq!(cmap.beta::<1>(DartId(24)), DartId(22));

    assert_eq!(cmap.beta::<2>(DartId(22)), DartId(20));
    assert_eq!(cmap.beta::<2>(DartId(23)), DartId(0));
    assert_eq!(cmap.beta::<2>(DartId(24)), DartId(0));
}
