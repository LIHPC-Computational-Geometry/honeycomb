use std::env;

use honeycomb_core::cmap::{CMap2, CMapBuilder};
use honeycomb_render::render_2d_map;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let map: CMap2<f32> = match CMapBuilder::<2>::from_vtk_file(path).build() {
            Ok(cmap) => cmap,
            Err(e) => panic!("Error while building map: {e:?}"),
        };

        render_2d_map(map);
    } else {
        println!(
            "No input file specified - you can pass a path to a vtk file as command line argument"
        )
    }
}
