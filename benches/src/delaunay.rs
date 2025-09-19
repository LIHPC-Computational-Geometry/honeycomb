use honeycomb::core::{cmap::CMap3, geometry::CoordsFloat};
use honeycomb::kernels::delaunay::delaunay_box_3d;

use crate::cli::DelaunayBoxArgs;

pub fn bench_delaunay<T: CoordsFloat>(args: DelaunayBoxArgs) -> CMap3<T> {
    let DelaunayBoxArgs {
        lx,
        ly,
        lz,
        n_points,
        alternate_init,
    } = args;
    let n_points = n_points.get();

    let (n_points_init, file_init) = if let Some(arg) = alternate_init {
        (
            arg.n_points_init.map(|v| v.get()).unwrap_or(0),
            arg.file_init,
        )
    } else {
        (0, None)
    };
    // let n_points_init = n_points_init.map(|v| v.get()).unwrap_or(0);

    delaunay_box_3d(lx, ly, lz, n_points, n_points_init, file_init)
}
