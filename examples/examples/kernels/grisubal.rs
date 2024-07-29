use honeycomb_kernels::grisubal;
use honeycomb_render::{RenderParameters, SmaaMode};

fn main() {
    let map = grisubal::<f64>("../../meshing-samples/vtk/2D/rectangle.vtk", (1., 1.), None);

    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        relative_resize: false,
        shrink_factor: 0.05,
        arrow_headsize: 0.01,
        arrow_thickness: 0.005,
    };

    honeycomb_render::launch(render_params, Some(&map));
}
