use honeycomb_render::{RenderParameters, Runner, SmaaMode};

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Disabled,
        ..Default::default()
    };
    Runner::default().run::<f32>(render_params, None);
}
