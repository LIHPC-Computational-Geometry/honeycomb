pub(crate) mod model;

// routines submodules
pub(crate) mod adaptive_grid;
pub(crate) mod dualization;
pub(crate) mod helpers;
pub(crate) mod regularisation;

use adaptive_grid::refinement;
use dualization::dualize_map;
use helpers::remove_dangling_darts;
pub use model::VtkError;
use model::{manual_grid, vtk_grid};
use regularisation::regularize_map;

use honeycomb::core::{cmap::CMap2, geometry::CoordsFloat};

const MAX_DEPTH: u32 = 10;

#[allow(clippy::needless_pass_by_value)]
/// Create an overlay grid from a VTK file input
pub fn overlay_grid<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path>,
    _grid_cell_sizes: [T; 2],
    nb_verts: Option<usize>,
    max_depth: Option<u32>,
) -> Result<CMap2<T>, VtkError> {
    // Determine whether to use VTK file or manual grid based on file path
    let file_path_str = file_path.as_ref().to_string_lossy();
    let (mut map, mut geo_verts) = if file_path_str == "random" || file_path_str.is_empty() {
        let verts_count = nb_verts.unwrap_or(10000);
        manual_grid::<T>(verts_count)
    } else {
        vtk_grid::<T>(&file_path_str)?
    };

    let depth = max_depth.unwrap_or(MAX_DEPTH);
    let t0 = std::time::Instant::now();
    refinement(&mut map, &mut geo_verts, depth);
    remove_dangling_darts(&mut map);
    let t_ref = t0.elapsed();
    println!("refinement took {:.3} s", t_ref.as_secs_f64());

    let t1 = std::time::Instant::now();
    regularize_map(&mut map);
    let t_reg = t1.elapsed();
    println!("regularize_map took {:.3} s", t_reg.as_secs_f64());

    let t2 = std::time::Instant::now();
    let dual_map = dualize_map(&map);
    let t_dual = t2.elapsed();
    println!("dualize_map took {:.3} s", t_dual.as_secs_f64());

    Ok(dual_map)
}
