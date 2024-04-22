use honeycomb_core::{
    utils::square_cmap2, CMap2, DartIdentifier, FloatType, Vector2, NULL_DART_ID,
};
use honeycomb_render::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    SeedableRng,
};

fn main() {
    const N_SQUARE: usize = 16;
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        ..Default::default()
    };
    let mut map: CMap2<FloatType> = square_cmap2(N_SQUARE);

    let seed: u64 = 9817498146784;
    let mut rngx = SmallRng::seed_from_u64(seed);
    let mut rngy = SmallRng::seed_from_u64(seed);
    let range: Uniform<FloatType> = Uniform::new(-0.5, 0.5);
    let xs = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngx));
    let ys = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngy));

    let offsets: Vec<Vector2<FloatType>> = xs.zip(ys).map(|(x, y)| (x, y).into()).collect();
    let n_offsets = offsets.len();

    let vertices = map.fetch_vertices();

    vertices.identifiers.iter().for_each(|vertex_id| {
        let neighbors_vertex_cell: Vec<DartIdentifier> = map
            .i_cell::<0>(*vertex_id as DartIdentifier)
            .map(|d_id| map.beta::<2>(d_id))
            .collect();
        if !neighbors_vertex_cell.contains(&NULL_DART_ID) {
            let current_value = map.vertex(*vertex_id);
            let _ = map.replace_vertex(
                *vertex_id,
                current_value + offsets[*vertex_id as usize % n_offsets],
            );
        }
    });

    Runner::default().run(render_params, Some(&map));
}
