use honeycomb_core::{utils::GridBuilder, CMap2};
use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let map: CMap2<f32> = GridBuilder::unit_squares(4).build2().unwrap();
    Runner::default().run(render_params, Some(&map));
}
