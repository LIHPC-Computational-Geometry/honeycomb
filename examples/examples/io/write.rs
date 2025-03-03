use std::time::Instant;

use honeycomb_core::cmap::{CMap2, CMapBuilder, DartIdType};
use rand::{
    SeedableRng,
    distr::{Bernoulli, Distribution},
    rngs::SmallRng,
};

fn main() {
    const N_SQUARE: usize = 16;
    const P_BERNOULLI: f64 = 0.6;

    // GENERATE THE MAP

    println!("I: Start map initialization...");
    let now = Instant::now();
    let mut map: CMap2<f32> = CMapBuilder::unit_grid(N_SQUARE).build().unwrap();
    let elapsed = now.elapsed();
    println!("I: Finished initializing in {}μs", elapsed.as_micros());

    let seed: u64 = 9817498146784;
    let rng = SmallRng::seed_from_u64(seed);
    let dist = Bernoulli::new(P_BERNOULLI).unwrap();
    let splits: Vec<bool> = dist.sample_iter(rng).take(N_SQUARE.pow(2)).collect();
    let n_split = splits.len();

    println!("I: Start quad split process...");
    let now = Instant::now();
    let faces: Vec<_> = map.iter_faces().collect();
    faces
        .iter()
        .filter(|square| splits[**square as usize % n_split])
        .for_each(|square| {
            let square = *square as DartIdType;
            let (d1, d2, d3, d4) = (square, square + 1, square + 2, square + 3);
            // in a parallel impl, we would create all new darts before-hand
            let dsplit1 = map.add_free_darts(2);
            let dsplit2 = dsplit1 + 1;
            // unsew the square & duplicate vertices to avoid data loss
            // this duplication effectively means that there are two existing vertices
            // for a short time, before being merged back by the sewing ops
            map.force_unsew::<1>(d1).unwrap();
            map.force_unsew::<1>(d3).unwrap();
            // link the two new dart in order to
            map.force_link::<2>(dsplit1, dsplit2).unwrap();
            // define beta1 of the new darts, i.e. tell them where they point to
            map.force_sew::<1>(dsplit1, d4).unwrap();
            map.force_sew::<1>(dsplit2, d2).unwrap();

            // sew the original darts to the new darts
            map.force_sew::<1>(d1, dsplit1).unwrap();
            map.force_sew::<1>(d3, dsplit2).unwrap();
            // fuse the edges; this is where duplicated vertices are merged back together
        });
    let elapsed = now.elapsed();
    println!("I: Finished splitting in {}μs", elapsed.as_micros());

    // SERIALIZE THE MAP

    println!("I: Start VTK file generation...");
    let now = Instant::now();

    let file = std::fs::File::create_new("splitsome.vtk").unwrap();
    map.to_vtk_binary(file);

    let elapsed = now.elapsed();
    println!(
        "I: Finished generating artifact in {}μs",
        elapsed.as_micros()
    );
}
