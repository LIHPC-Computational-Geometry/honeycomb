use honeycomb_core::{CMap2, CMapBuilder, GridDescriptor};
use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let grid_descriptor = GridDescriptor::default()
        .n_cells([15, 5, 0])
        .len_per_cell_x(1.0_f64)
        .len_per_cell_y(3.0_f64);
    let map: CMap2<f64> = CMapBuilder::default()
        .grid_descriptor(grid_descriptor)
        .build()
        .unwrap();
    launch(render_params, Some(&map));
}
