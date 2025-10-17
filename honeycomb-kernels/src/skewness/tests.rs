use honeycomb_core::cmap::CMap2;

use crate::{grid_generation::GridBuilder, skewness::compute_face_skewness_2d};

#[allow(clippy::float_cmp)]
#[test]
fn dim2_grids() {
    // squares are equiangular
    let map: CMap2<f32> = GridBuilder::<2, f32>::unit_grid(2);
    assert!(
        map.iter_faces()
            .map(|fid| compute_face_skewness_2d(&map, fid))
            .all(|s| s == 0.0)
    );
    // triangles aren't; their angles are 90, 45, 45
    let map: CMap2<f32> = GridBuilder::<2, f32>::unit_triangles(2);
    assert!(
        map.iter_faces()
            .map(|fid| compute_face_skewness_2d(&map, fid))
            .all(|s| s == 0.25)
    );
}
