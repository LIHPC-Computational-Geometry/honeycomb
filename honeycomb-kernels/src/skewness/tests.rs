use honeycomb_core::cmap::{CMap2, CMapBuilder};

use crate::skewness::compute_faces_skewness;

#[test]
fn skewtest() {
    // squares are equiangular
    let map: CMap2<f32> = CMapBuilder::unit_grid(2).build().unwrap();
    assert!(compute_faces_skewness(&map).all(|s| s == 0.0));
    // triangles aren't; their angles are 90, 45, 45
    let map: CMap2<f32> = CMapBuilder::unit_triangles(2).build().unwrap();
    assert!(compute_faces_skewness(&map).all(|s| s == 0.25));
}
