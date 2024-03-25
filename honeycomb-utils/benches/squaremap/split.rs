//! This benchmarks handle measurements for a given operation on CMap2
//! of a given topology (see `generation::splitsquare_two_map` doc).
//!
//! The operations applied here affect only topology, cells is left unchanged
//! aside from the new faces divisiosns.
//!
//! All splitting operations consist in splitting diagonally a square face to create
//! two triangular faces; The specifities are as follow:
//!
//! - the `split` routine performs the splits uniformly across all squares making up
//!   the map.
//! - the `split_some` routine performs the splits uniformly only on certain squares
//!   of the map. Whether a square is split or not is determined by a Bernoulli distribution.
//! - the `split_diff` routine performs the splits along one diagonal or the other, across
//!   all squares of the map. Which way a square is split is determined by a Bernoulli
//!   distribution.
//!

// ------ IMPORTS

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{
    distributions::{Bernoulli, Distribution},
    rngs::SmallRng,
    SeedableRng,
};

use honeycomb_core::{CMap2, DartIdentifier, FloatType, SewPolicy, UnsewPolicy};
use honeycomb_utils::generation::square_cmap2;

// ------ CONTENT

const N_SQUARE: usize = 2_usize.pow(10);
const P_BERNOULLI: f64 = 0.6;

fn split(mut map: CMap2<FloatType>) {
    (0..N_SQUARE.pow(2)).for_each(|square| {
        let d1 = (1 + square * 4) as DartIdentifier;
        let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
        // in a parallel impl, we would create all new darts before-hand
        let dsplit1 = map.add_free_darts(2);
        let dsplit2 = dsplit1 + 1;
        map.two_sew(dsplit1, dsplit2, SewPolicy::StretchLeft);
        map.one_unsew(d1, UnsewPolicy::DoNothing);
        map.one_unsew(d3, UnsewPolicy::DoNothing);
        map.one_sew(d1, dsplit1, SewPolicy::StretchLeft);
        map.one_sew(d3, dsplit2, SewPolicy::StretchLeft);
        map.one_sew(dsplit1, d4, SewPolicy::StretchRight);
        map.one_sew(dsplit2, d2, SewPolicy::StretchRight);
    });

    // rebuild faces
    assert_eq!(map.build_all_faces(), N_SQUARE.pow(2) * 2);

    black_box(&mut map);
}

fn split_some(mut map: CMap2<FloatType>, split: &[bool]) {
    (0..N_SQUARE.pow(2))
        .filter(|square| split[*square]) // split only if true
        .for_each(|square| {
            let d1 = (1 + square * 4) as DartIdentifier;
            let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
            // in a parallel impl, we would create all new darts before-hand
            let dsplit1 = map.add_free_darts(2);
            let dsplit2 = dsplit1 + 1;
            map.two_sew(dsplit1, dsplit2, SewPolicy::StretchLeft);
            map.one_unsew(d1, UnsewPolicy::DoNothing);
            map.one_unsew(d3, UnsewPolicy::DoNothing);
            map.one_sew(d1, dsplit1, SewPolicy::StretchLeft);
            map.one_sew(d3, dsplit2, SewPolicy::StretchLeft);
            map.one_sew(dsplit1, d4, SewPolicy::StretchRight);
            map.one_sew(dsplit2, d2, SewPolicy::StretchRight);
        });

    // rebuild faces
    map.build_all_faces();

    black_box(&mut map);
}

fn split_diff(mut map: CMap2<FloatType>, split: &[bool]) {
    (0..N_SQUARE.pow(2)).for_each(|square| {
        let ddown = (1 + square * 4) as DartIdentifier;
        let (dright, dup, dleft) = (ddown + 1, ddown + 2, ddown + 3);
        // in a parallel impl, we would create all new darts before-hand
        let dsplit1 = map.add_free_darts(2);
        let dsplit2 = dsplit1 + 1;
        // this leads to the same result as the commented code below; there doesn't seem
        // to be any change in performances
        let (dbefore1, dbefore2, dafter1, dafter2) = if split[square] {
            (ddown, dup, dleft, dright)
        } else {
            (dright, dleft, ddown, dup)
        };
        map.two_sew(dsplit1, dsplit2, SewPolicy::StretchLeft);
        map.one_unsew(dbefore1, UnsewPolicy::DoNothing);
        map.one_unsew(dbefore2, UnsewPolicy::DoNothing);
        map.one_sew(dbefore1, dsplit1, SewPolicy::StretchLeft);
        map.one_sew(dbefore2, dsplit2, SewPolicy::StretchLeft);
        map.one_sew(dsplit1, dafter1, SewPolicy::StretchRight);
        map.one_sew(dsplit2, dafter2, SewPolicy::StretchRight);
        /*
        if split[square] {
            map.one_unsew(ddown, UnsewPolicy::DoNothing);
            map.one_unsew(dup, UnsewPolicy::DoNothing);
            map.one_sew(ddown, dsplit1, SewPolicy::StretchLeft);
            map.one_sew(dup, dsplit2, SewPolicy::StretchLeft);
            map.one_sew(dsplit1, dleft, SewPolicy::StretchRight);
            map.one_sew(dsplit2, dright, SewPolicy::StretchRight);
        } else {
            map.one_unsew(dright, UnsewPolicy::DoNothing);
            map.one_unsew(dleft, UnsewPolicy::DoNothing);
            map.one_sew(dright, dsplit1, SewPolicy::StretchLeft);
            map.one_sew(dleft, dsplit2, SewPolicy::StretchLeft);
            map.one_sew(dsplit1, ddown, SewPolicy::StretchRight);
            map.one_sew(dsplit2, dup, SewPolicy::StretchRight);
        }*/
    });

    // rebuild faces
    assert_eq!(map.build_all_faces(), N_SQUARE.pow(2) * 2);

    black_box(&mut map);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let map: CMap2<FloatType> = square_cmap2(N_SQUARE);
    let seed: u64 = 9817498146784;
    let rng = SmallRng::seed_from_u64(seed);
    let dist = Bernoulli::new(P_BERNOULLI).unwrap();
    let splits: Vec<bool> = dist.sample_iter(rng).take(N_SQUARE.pow(2)).collect();

    let mut group = c.benchmark_group("squaremap-split");

    group.bench_with_input(
        BenchmarkId::new("all-diagonals", ""),
        &map.clone(),
        |b, map| b.iter(|| split(map.clone())),
    );
    group.bench_with_input(
        BenchmarkId::new("some-diagonals", ""),
        &(map.clone(), splits.clone()),
        |b, (map, splits)| b.iter(|| split_some(map.clone(), splits)),
    );
    group.bench_with_input(
        BenchmarkId::new("diff-diagonals", ""),
        &(map.clone(), splits.clone()),
        |b, (map, splits)| b.iter(|| split_diff(map.clone(), splits)),
    );

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
