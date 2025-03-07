use std::env;

use honeycomb_core::cmap::{CMap2, CMapBuilder};
use honeycomb_render::App;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let map: CMap2<f32> = match CMapBuilder::<2, f32>::from_vtk_file(path).build() {
            Ok(cmap) => cmap,
            Err(e) => panic!("Error while building map: {e:?}"),
        };

        let mut app = App::default();
        app.add_capture(&map);
        app.run()
    } else {
        println!(
            "No input file specified - you can pass a path to a vtk file as command line argument"
        )
    }
}
