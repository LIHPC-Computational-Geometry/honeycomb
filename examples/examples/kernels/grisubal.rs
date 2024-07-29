use honeycomb_kernels::grisubal;
use honeycomb_render::{RenderParameters, SmaaMode};

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let map = grisubal::<f64>(path, (1., 1.), None);

        let render_params = RenderParameters {
            smaa_mode: SmaaMode::Smaa1X,
            relative_resize: false,
            shrink_factor: 0.05,
            arrow_headsize: 0.01,
            arrow_thickness: 0.005,
        };

        honeycomb_render::launch(render_params, Some(&map));
    } else {
        println!("No input geometry specified - you can pass a path to a vtk input as command line argument")
    }
}
