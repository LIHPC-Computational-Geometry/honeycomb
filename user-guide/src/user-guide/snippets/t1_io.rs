use honeycomb::prelude::*;

pub fn main() {
    // import the custom `.cmap` format (2D and 3D maps)
    let map: CMap2<f32> = CMapBuilder::<2>::from_cmap_file("mesh.cmap")
        .build()
        .unwrap();

    // export to VTK (2D maps only)
    let file = std::fs::File::create("out.vtk").unwrap();
    map.to_vtk_binary(file);

    // visual debugging (requires the `render` feature)
    render_2d_map(map);
}
