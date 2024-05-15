use honeycomb_core::CMap2;
use honeycomb_render::*;

fn main() {
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        relative_resize: true,
        shrink_factor: 0.03,
        ..Default::default()
    };
    let map: CMap2<f64> = CMap2::from_vtk_file("honeycomb-examples/examples/io/tri.vtk");
    assert_eq!(map.fetch_vertices().identifiers.len(), 8);
    assert_eq!(map.fetch_edges().identifiers.len(), 15);
    assert_eq!(map.fetch_faces().identifiers.len(), 8);
    launch(render_params, Some(&map));
}
