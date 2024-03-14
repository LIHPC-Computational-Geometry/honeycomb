use honeycomb_core::TwoMap;
use honeycomb_render::*;
use honeycomb_utils::generation::splitsquare_two_map;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let map: TwoMap<1, f32> = splitsquare_two_map(4);
    Runner::default().run(render_params, Some(&map));
}
