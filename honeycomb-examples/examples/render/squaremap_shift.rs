use honeycomb_core::{utils::GridBuilder, CMap2, DartIdentifier, FloatType, Vector2, NULL_DART_ID};
use honeycomb_render::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
};
use std::time::Instant;

fn main() {
    const N_SQUARE: usize = 16;
    let render_params = RenderParameters {
        smaa_mode: SmaaMode::Smaa1X,
        relative_resize: false,
        ..Default::default()
    };

    println!("I: Start map initialization...");
    let now = Instant::now();
    let mut map: CMap2<FloatType> = GridBuilder::unit_squares(N_SQUARE).build2().unwrap();
    let elapsed = now.elapsed();
    println!("I: Finished initializing in {}μs", elapsed.as_micros());

    println!("I: Start offset generation...");
    let now = Instant::now();
    let seed: u64 = 9817498146784;
    let mut rngx = SmallRng::seed_from_u64(seed);
    let mut rngy = SmallRng::seed_from_u64(seed);
    let range: Uniform<FloatType> =
        Uniform::new(-0.5, 0.5).expect("Could not initialize the uniform distribution");
    let xs = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngx));
    let ys = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngy));
    let offsets: Vec<Vector2<FloatType>> = xs.zip(ys).map(|(x, y)| (x, y).into()).collect();
    let n_offsets = offsets.len();
    let elapsed = now.elapsed();
    println!(
        "I: Finished generating offsets in {}μs",
        elapsed.as_micros()
    );

    println!("I: Start shifting inner vertices...");
    let now = Instant::now();
    map.fetch_vertices()
        .identifiers
        .iter()
        .for_each(|vertex_id| {
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
    let elapsed = now.elapsed();
    println!("I: Finished shifting in {}μs", elapsed.as_micros());

    Runner::default().run(render_params, Some(&map));
}
