use honeycomb_kernels::grid_generation::GridBuilder;
use honeycomb_render::render_2d_map;

fn main() {
    // build a simple 4 by 4 grid at origin (1.5, 1.5)

    let map = GridBuilder::<2, f32>::default()
        .origin([1.5, 1.5])
        .n_cells([4, 4])
        .len_per_cell([1., 1.])
        .build()
        .unwrap();

    // create the render app, add the map to it, and run the app

    render_2d_map(map);
}
