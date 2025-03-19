use std::env;

use honeycomb_kernels::grisubal::*;
use honeycomb_render::render_2d_map;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let clip = if let Some(val) = args.get(2) {
            match val.as_ref() {
                "left" => Clip::Left,
                "right" => Clip::Right,
                _ => {
                    eprintln!("W: unrecognised clip argument - running kernel without clipping");
                    Clip::None
                }
            }
        } else {
            Clip::None
        };

        let map = grisubal::<f64>(path, [1., 1.], clip).unwrap();

        render_2d_map(map);
    } else {
        println!(
            "No input geometry specified - you can pass a path to a vtk input as command line argument"
        )
    }
}
