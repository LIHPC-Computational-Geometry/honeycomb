use honeycomb::prelude::{CMapBuilder, GridDescriptor};

fn main() {
    // ./binary nx ny split
    let grid = {
        let args: Vec<String> = std::env::args().collect();
        match (args.get(1), args.get(2), args.get(3)) {
            (Some(nx), Some(ny), split) => {
                let nx = nx.parse().unwrap();
                let ny = ny.parse().unwrap();
                GridDescriptor::default()
                    .n_cells([nx, ny, 0])
                    .lens([1.0, 1.0, 1.0])
                    .split_quads(split.is_some())
            }
            _ => panic!("E: specify at least the number of cells along X and Y axes"),
        }
    };

    let map = CMapBuilder::default()
        .grid_descriptor(grid)
        .build()
        .unwrap();

    std::hint::black_box(map);
}
