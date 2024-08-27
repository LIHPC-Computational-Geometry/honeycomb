use honeycomb_core::{CMap2, CMapBuilder};

fn main() {
    let cmap: CMap2<f32> = CMapBuilder::unit_grid(4).build().unwrap();
    let mut app = honeycomb_render::App::default();
    app.add_capture(&cmap);
    app.run()
}
