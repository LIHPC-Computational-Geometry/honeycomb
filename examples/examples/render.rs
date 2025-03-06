use honeycomb_core::cmap::{CMapBuilder, GridDescriptor};
use honeycomb_core::geometry::Vertex2;
use honeycomb_render::App;

fn main() {
    // build a simple 4 by 4 grid at origin (1.5, 1.5)

    let map = CMapBuilder::default()
        .grid_descriptor(
            GridDescriptor::default()
                .origin(Vertex2(1.5, 1.5))
                .n_cells_x(4)
                .n_cells_y(4)
                .len_per_cell_x(1.)
                .len_per_cell_y(1.),
        )
        .build()
        .unwrap();

    // create the render app, add the map to it, and run the app

    let mut render_app = App::default();
    render_app.add_capture(&map);
    render_app.run()
}
