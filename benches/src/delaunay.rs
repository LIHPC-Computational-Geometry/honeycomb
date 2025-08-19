use honeycomb_core::{
    cmap::{CMap3, CMapBuilder},
    geometry::{CoordsFloat, Vertex3},
    stm::TVar,
};

pub fn delaunay_box<T: CoordsFloat>(len_x: T, len_y: T, len_z: T, n_points: usize) -> CMap3<T> {
    assert!(len_x.is_sign_positive() && !len_x.is_zero());
    assert!(len_y.is_sign_positive() && !len_y.is_zero());
    assert!(len_z.is_sign_positive() && !len_z.is_zero());
    // assert!(n_points > 5); // or check the validity of the box later? volume>0?

    // TODO: Sample points in the [0;len_x]x[0;len_y]x[0;len_z] bounding box, build the actual box
    let points: Vec<Vertex3<T>> = Vec::with_capacity(n_points);
    let mut map = CMapBuilder::<3, T>::from_n_darts(60)
        .build()
        .expect("E: unreachable");

    // TODO: process points until all are inserted
    points.into_iter().for_each(|p| {
        // locate
        // compute cavity
        // carve
        // rebuild
    });

    map
}
