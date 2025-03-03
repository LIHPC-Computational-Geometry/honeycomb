use honeycomb_core::cmap::{CMap2, CMapBuilder};

use crate::skewness::compute_face_skewness_2d;

#[test]
fn dim2_grids() {
    // squares are equiangular
    let map: CMap2<f32> = CMapBuilder::unit_grid(2).build().unwrap();
<<<<<<< HEAD
    assert!(
        map.iter_faces()
            .map(|fid| compute_face_skewness_2d(&map, fid))
            .all(|s| s == 0.0)
    );
    // triangles aren't; their angles are 90, 45, 45
    let map: CMap2<f32> = CMapBuilder::unit_triangles(2).build().unwrap();
    assert!(
        map.iter_faces()
            .map(|fid| compute_face_skewness_2d(&map, fid))
            .all(|s| s == 0.25)
    );
=======
    assert!(compute_cells_skewness_2d(&map).all(|s| s == 0.0));
    // triangles aren't; their angles are 90, 45, 45
    let map: CMap2<f32> = CMapBuilder::unit_triangles(2).build().unwrap();
    assert!(compute_cells_skewness_2d(&map).all(|s| s == 0.25));
>>>>>>> 69906560083ca04217e52dcd1a55a67ff2a4206f
}
