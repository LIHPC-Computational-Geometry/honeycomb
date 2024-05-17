use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    launch::<f32>(render_params, None);
}
