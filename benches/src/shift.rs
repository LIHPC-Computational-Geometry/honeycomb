use honeycomb::{
    kernels::shift::shift_vertices_to_neigh_avg,
    prelude::{CMap2, CMapBuilder},
};

fn main() {
    // ./binary ~/path/to/file.vtk n_rounds
    let args: Vec<String> = std::env::args().collect();
    let path = if let Some(path) = args.get(1) {
        path.clone()
    } else {
        "examples/quads.vtk".to_string()
    };
    let n_rounds = args
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    let map: CMap2<f64> = CMapBuilder::default().vtk_file(path).build().unwrap();

    shift_vertices_to_neigh_avg(&map, n_rounds);

    std::hint::black_box(map);
}
