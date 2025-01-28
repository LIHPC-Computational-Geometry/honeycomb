use std::{fs::File, time::Instant};

use honeycomb::{
    core::{
        cmap::CMapError,
        stm::{atomically, StmError},
    },
    prelude::{CMap2, CMapBuilder, DartIdType, Orbit2, OrbitPolicy},
};

use rand::{
    distr::{Bernoulli, Distribution},
    rngs::SmallRng,
    SeedableRng,
};

use rayon::prelude::*;

macro_rules! process_unsew {
    ($op: expr) => {
        if let Err(e) = $op {
            return match e {
                CMapError::FailedTransaction(e) => Err(e),
                CMapError::FailedAttributeSplit(_) => Err(StmError::Retry),
                CMapError::FailedAttributeMerge(_)
                | CMapError::IncorrectGeometry(_)
                | CMapError::UnknownAttribute(_) => unreachable!(),
            };
        }
    };
}

macro_rules! process_sew {
    ($op: expr) => {
        if let Err(e) = $op {
            return match e {
                CMapError::FailedTransaction(e) => Err(e),
                CMapError::FailedAttributeMerge(_) => Err(StmError::Retry),
                CMapError::FailedAttributeSplit(_)
                | CMapError::IncorrectGeometry(_)
                | CMapError::UnknownAttribute(_) => unreachable!(),
            };
        }
    };
}
const N_SQUARE: usize = 128;
const N_DIAG: usize = N_SQUARE.pow(2);
const P_BERNOULLI: f64 = 0.6;
const SEED: u64 = 9_817_498_146_784;

fn main() {
    let mut instant = Instant::now();
    // (a) generate the grid
    let mut map: CMap2<f64> = CMapBuilder::unit_grid(N_SQUARE).build().unwrap();
    // collect these now so additional darts don't mess up the iterator
    let faces = map.iter_faces().collect::<Vec<_>>();
    println!("grid built in {}ms", instant.elapsed().as_millis());

    // (b) split diagonals one way or the other
    // sample diag orientation
    let rng = SmallRng::seed_from_u64(SEED);
    let dist = Bernoulli::new(P_BERNOULLI).unwrap();
    let splits: Vec<bool> = dist.sample_iter(rng).take(N_DIAG).collect();
    // allocate diag darts
    let nd = map.add_free_darts(N_DIAG * 2);
    let nd_range = (nd..nd + (N_DIAG * 2) as DartIdType).collect::<Vec<_>>();

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
            map.force_link::<2>(dsplit1, dsplit2);

            atomically(|trans| {
                process_unsew!(map.unsew::<1>(trans, dbefore1));
                process_unsew!(map.unsew::<1>(trans, dbefore2));
                process_sew!(map.sew::<1>(trans, dsplit1, dafter1));
                process_sew!(map.sew::<1>(trans, dsplit2, dafter2));

                process_sew!(map.sew::<1>(trans, dbefore1, dsplit1));
                process_sew!(map.sew::<1>(trans, dbefore2, dsplit2));
                Ok(())
            });
        });
    println!("diagonals split in {}ms", instant.elapsed().as_millis());

    instant = Instant::now();
    // (c) save the map
    let mut f = File::create("grid_split.vtk").unwrap();
    map.to_vtk_binary(&mut f);
    println!("map saved in {}ms", instant.elapsed().as_millis());
}
