use honeycomb_core::{
    cmap::{CMapBuilder, OrbitPolicy},
    stm::atomically_with_err,
};

use crate::remeshing::{EdgeSwapError, swap_edge};

// -- collapse

#[test]
fn collapse_edge_errs() {
    // call on null

    // quad on one side

    // quad on the other
}

#[test]
fn collapse_edge_seq() {}

// -- swap

#[test]
fn swap_edge_errs() {
    let map = CMapBuilder::<2, f64>::unit_triangles(1).build().unwrap();

    assert!(
        atomically_with_err(|t| swap_edge(t, &map, 0)).is_err_and(|e| e == EdgeSwapError::NullEdge)
    );
    assert!(
        atomically_with_err(|t| swap_edge(t, &map, 1))
            .is_err_and(|e| e == EdgeSwapError::IncompleteEdge)
    );

    let map = CMapBuilder::<2, f64>::unit_grid(2).build().unwrap();

    assert!(
        atomically_with_err(|t| swap_edge(t, &map, 2))
            .is_err_and(|e| e == EdgeSwapError::BadTopology)
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
