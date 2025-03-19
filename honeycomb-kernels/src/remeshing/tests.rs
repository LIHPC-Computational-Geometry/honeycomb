use honeycomb_core::{
    cmap::{CMapBuilder, OrbitPolicy},
    stm::atomically_with_err,
};

use crate::remeshing::{EdgeSwapError, swap_edge};

#[test]
fn swap_edge_boundary() {
    let map = CMapBuilder::<2, f64>::unit_triangles(1).build().unwrap();

    assert!(
        atomically_with_err(|t| swap_edge(t, &map, 1))
            .is_err_and(|e| e == EdgeSwapError::IncompleteEdge)
    );
}

#[test]
fn swap_edge_seq() {
    let map = CMapBuilder::<2, f64>::unit_triangles(1).build().unwrap();

    // before
    //
    // +---+
    // |\  |
    // | \ |
    // |  \|
    // +---+

    let o1: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 1).collect();
    assert_eq!(&o1, &[1, 2, 3]);
    let o6: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 6).collect();
    assert_eq!(&o6, &[6, 4, 5]);

    assert!(atomically_with_err(|t| swap_edge(t, &map, 2)).is_ok());

    // after
    //
    // +---+
    // |  /|
    // | / |
    // |/  |
    // +---+

    let o1: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 1).collect();
    assert_eq!(&o1, &[1, 5, 4]);
    let o6: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 6).collect();
    assert_eq!(&o6, &[6, 3, 2]);
}
