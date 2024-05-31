use honeycomb_core::{CMap2, CMapBuilder};
use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        relative_resize: true,
        shrink_factor: 0.01,
        ..Default::default()
    };
    let map: CMap2<f64> = CMapBuilder::from_vtk_file("assets/quad.vtk")
        .build()
        .unwrap();
    assert_eq!(map.fetch_vertices().identifiers.len(), 21);
    assert_eq!(map.fetch_edges().identifiers.len(), 32);
    assert_eq!(map.fetch_faces().identifiers.len(), 12);
    launch(render_params, Some(&map));
}
