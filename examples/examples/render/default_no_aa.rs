use honeycomb_render::{launch, RenderParameters, SmaaMode};

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Disabled,
        ..Default::default()
    };
    launch::<f32>(render_params, None);
}
