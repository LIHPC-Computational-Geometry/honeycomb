use std::{fs::File, time::Instant};

use honeycomb::{
    core::stm::atomically_with_err,
    prelude::{CMap2, CMapBuilder, DartIdType, Orbit2, OrbitPolicy},
};

use rand::{
    distr::{Bernoulli, Distribution},
    rngs::SmallRng,
    SeedableRng,
};

use rayon::prelude::*;

const P_BERNOULLI: f64 = 0.6;
const SEED: u64 = 9_817_498_146_784;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n_square = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(128);
    let n_diag = n_square.pow(2);
    let mut instant = Instant::now();
    // (a) generate the grid
    let mut map: CMap2<f64> = CMapBuilder::unit_grid(n_square).build().unwrap();
    // collect these now so additional darts don't mess up the iterator
    let faces = map.iter_faces().collect::<Vec<_>>();
    println!("grid built in {}ms", instant.elapsed().as_millis());

    // (b) split diagonals one way or the other
    // sample diag orientation
    let rng = SmallRng::seed_from_u64(SEED);
    let dist = Bernoulli::new(P_BERNOULLI).unwrap();
    let splits: Vec<bool> = dist.sample_iter(rng).take(n_diag).collect();
    // allocate diag darts
    let nd = map.add_free_darts(n_diag * 2);
    let nd_range = (nd..nd + (n_diag * 2) as DartIdType).collect::<Vec<_>>();

    instant = Instant::now();
    // build diags
    faces
        .into_iter()
        .zip(nd_range.chunks(2))
        .zip(splits.into_iter())
        .par_bridge()
        .for_each(|((df, sl), split)| {
            let square = df as DartIdType;
            assert_eq!(Orbit2::new(&map, OrbitPolicy::FaceLinear, df).count(), 4);
            let (ddown, dright, dup, dleft) = (square, square + 1, square + 2, square + 3);

            let &[dsplit1, dsplit2] = sl else {
                unreachable!()
            };
            let (dbefore1, dbefore2, dafter1, dafter2) = if split {
                (ddown, dup, dleft, dright)
            } else {
                (dright, dleft, ddown, dup)
            };
            let _ = map.force_link::<2>(dsplit1, dsplit2);

            while atomically_with_err(|trans| {
                map.unsew::<1>(trans, dbefore1)?;
                map.unsew::<1>(trans, dbefore2)?;
                map.sew::<1>(trans, dsplit1, dafter1)?;
                map.sew::<1>(trans, dsplit2, dafter2)?;

                map.sew::<1>(trans, dbefore1, dsplit1)?;
                map.sew::<1>(trans, dbefore2, dsplit2)?;
                Ok(())
            })
            .is_err()
            {}
        });
    println!("diagonals split in {}ms", instant.elapsed().as_millis());

    instant = Instant::now();
    // (c) save the map
    let mut f = File::create("grid_split.vtk").unwrap();
    map.to_vtk_binary(&mut f);
    println!("map saved in {}ms", instant.elapsed().as_millis());
}
