use honeycomb_core::{CMap2, CMapBuilder};
use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let map: CMap2<f32> = CMapBuilder::unit_triangles(4).build().unwrap();
    launch(render_params, Some(&map));
}
