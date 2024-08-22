use honeycomb_core::{CMap2, CMapBuilder};

fn main() {
    let cmap: CMap2<f32> = CMapBuilder::unit_grid(4).build().unwrap();
    honeycomb_render::App::default().add_capture(&cmap).run();
}
