use honeycomb_core::CMap2;
use honeycomb_render::*;
use honeycomb_utils::generation::square_two_map;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let map: CMap2<1, f32> = square_two_map(4);
    Runner::default().run(render_params, Some(&map));
}
