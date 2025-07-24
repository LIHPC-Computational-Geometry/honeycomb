//! Grid generation benchmark
//!
//! This benchmark generates a grid using the passed parameters as argument for [`GridDescriptor`]
//! and [`CMapBuilder`].

use std::time::{Duration, Instant};

use honeycomb::{
    core::stm::atomically_with_err,
    prelude::{CMap2, CMapBuilder, CoordsFloat, DartIdType, GridDescriptor, OrbitPolicy},
};
use rand::{
    SeedableRng,
    distr::{Bernoulli, Distribution},
    rngs::SmallRng,
};
use rayon::prelude::*;

use crate::cli::{Generate2dGridArgs, Split};

pub fn bench_generate_2d_grid<T: CoordsFloat>(args: Generate2dGridArgs) -> CMap2<T> {
    let descriptor = GridDescriptor::<2, T>::default()
        .n_cells([args.nx.get(), args.ny.get()])
        .len_per_cell([T::from(args.lx).unwrap(), T::from(args.ly).unwrap()])
        .split_cells(args.split.is_some_and(|s| s == Split::Uniform));

    let mut map = CMapBuilder::from_grid_descriptor(descriptor)
        .build()
        .unwrap();

    if args.split.is_some_and(|s| s == Split::Random) {
        // fixed probability and seed value from  the original binary
        // this can be made into a CL arg if necessary
        let _ = split_faces_randomly(&mut map, 0.6, 9_817_498_146_784);
    }

    map
}

fn split_faces_randomly<T: CoordsFloat>(
    map: &mut CMap2<T>,
    p_bernoulli: f64,
    seed: u64,
) -> (Duration, Duration) {
    // sample
    let mut instant = Instant::now();
    let faces = map.iter_faces().collect::<Vec<_>>();
    let n_diag = faces.len();
    let rng = SmallRng::seed_from_u64(seed);
    let dist = Bernoulli::new(p_bernoulli).unwrap();
    let splits: Vec<bool> = dist.sample_iter(rng).take(n_diag).collect();
    let sample_time = instant.elapsed();

    // build diags
    instant = Instant::now();
    let nd = map.allocate_used_darts(n_diag * 2);
    let nd_range = (nd..nd + (n_diag * 2) as DartIdType).collect::<Vec<_>>();
    faces
        .into_iter()
        .zip(nd_range.chunks(2))
        .zip(splits)
        .par_bridge()
        .for_each(|((df, sl), split)| {
            let square = df as DartIdType;
            assert_eq!(map.orbit(OrbitPolicy::FaceLinear, df).count(), 4);
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

            while atomically_with_err(|t| {
                map.unsew::<1>(t, dbefore1)?;
                map.unsew::<1>(t, dbefore2)?;
                map.sew::<1>(t, dsplit1, dafter1)?;
                map.sew::<1>(t, dsplit2, dafter2)?;

                map.sew::<1>(t, dbefore1, dsplit1)?;
                map.sew::<1>(t, dbefore2, dsplit2)?;
                Ok(())
            })
            .is_err()
            {}
        });
    (sample_time, instant.elapsed())
}
