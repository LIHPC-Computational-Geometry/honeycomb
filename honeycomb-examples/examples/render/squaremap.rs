use honeycomb_core::{utils::square_cmap2, CMap2};
use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let map: CMap2<f32> = square_cmap2(4);
    Runner::default().run(render_params, Some(&map));
}
