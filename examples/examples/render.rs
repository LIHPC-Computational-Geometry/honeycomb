use honeycomb_core::cmap::{CMapBuilder, GridDescriptor};
use honeycomb_render::render_2d_map;

fn main() {
    // build a simple 4 by 4 grid at origin (1.5, 1.5)

    let map = CMapBuilder::<2, f64>::from_grid_descriptor(
        GridDescriptor::default()
            .origin([1.5, 1.5])
            .n_cells([4, 4])
            .len_per_cell([1., 1.]),
    )
    .build()
    .unwrap();

    // create the render app, add the map to it, and run the app

    render_2d_map(map);
}
