use honeycomb_core::{utils::GridBuilder, CMap2};
use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let map: CMap2<f64> = GridBuilder::default()
        .n_cells([15, 5, 0])
        .len_per_cell_x(1.0_f64)
        .len_per_cell_y(3.0_f64)
        .build2();
    Runner::default().run(render_params, Some(&map));
}
