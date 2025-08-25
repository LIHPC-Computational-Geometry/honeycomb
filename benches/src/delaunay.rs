use honeycomb::core::{cmap::CMap3, geometry::CoordsFloat};
use honeycomb::kernels::delaunay::delaunay_box_3d;

use crate::cli::DelaunayBoxArgs;

pub fn bench_delaunay<T: CoordsFloat>(args: DelaunayBoxArgs) -> CMap3<T> {
    let DelaunayBoxArgs {
        lx,
        ly,
        lz,
        n_points,
    } = args;
    let n_points = n_points.get();

    delaunay_box_3d(lx, ly, lz, n_points)
}
