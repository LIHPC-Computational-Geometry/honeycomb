use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    Runner::default().run::<f32>(render_params, None);
}
