use honeycomb_core::{utils::GridBuilder, CMap2, DartIdentifier};
use rand::distributions::Bernoulli;
use rand::{distributions::Distribution, rngs::SmallRng};
use std::time::Instant;

fn main() {
    const N_SQUARE: usize = 16;
    const P_BERNOULLI: f64 = 0.6;

    println!("I: Start map initialization...");
    let now = Instant::now();
    let mut map: CMap2<f32> = GridBuilder::unit_squares(N_SQUARE).build2().unwrap();
    let elapsed = now.elapsed();
    println!("I: Finished initializing in {}μs", elapsed.as_micros());

    let seed: u64 = 9817498146784;
    let rng = SmallRng::seed_from_u64(seed);
    let dist = Bernoulli::new(P_BERNOULLI).unwrap();
    let splits: Vec<bool> = dist.sample_iter(rng).take(N_SQUARE.pow(2)).collect();
    let n_split = splits.len();

    println!("I: Start quad split process...");
    let now = Instant::now();
    map.fetch_faces()
        .identifiers
        .iter()
        .filter(|square| splits[**square as usize % n_split])
        .for_each(|square| {
            let square = *square as DartIdentifier;
            let (d1, d2, d3, d4) = (square, square + 1, square + 2, square + 3);
            // in a parallel impl, we would create all new darts before-hand
            let dsplit1 = map.add_free_darts(2);
            let dsplit2 = dsplit1 + 1;
            // unsew the square & duplicate vertices to avoid data loss
            // this duplication effectively means that there are two existing vertices
            // for a short time, before being merged back by the sewing ops
            map.one_unsew(d1);
            map.one_unsew(d3);
            // link the two new dart in order to
            map.two_link(dsplit1, dsplit2);
            // define beta1 of the new darts, i.e. tell them where they point to
            map.one_sew(dsplit1, d4);
            map.one_sew(dsplit2, d2);

            // sew the original darts to the new darts
            map.one_sew(d1, dsplit1);
            map.one_sew(d3, dsplit2);
            // fuse the edges; this is where duplicated vertices are merged back together
        });
    let elapsed = now.elapsed();
    println!("I: Finished splitting in {}μs", elapsed.as_micros());

    println!("I: Start VTK file generation...");
    let now = Instant::now();

    let file = std::fs::File::create_new("splitsome.vtk").unwrap();
    map.to_vtk_file(file);

    let elapsed = now.elapsed();
    println!(
        "I: Finished generating artifact in {}μs",
        elapsed.as_micros()
    );
}
