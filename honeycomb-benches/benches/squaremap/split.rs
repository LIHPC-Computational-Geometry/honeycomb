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
use honeycomb_core::{utils::GridBuilder, CMap2, DartIdentifier, FloatType};
use rand::{
    distributions::{Bernoulli, Distribution},
    rngs::SmallRng,
};

// ------ CONTENT

const N_SQUARE: usize = 2_usize.pow(10);
const P_BERNOULLI: f64 = 0.6;

fn split(mut map: CMap2<FloatType>) {
    map.fetch_faces()
        .identifiers
        .iter()
        .copied()
        .for_each(|square| {
            let square = square as DartIdentifier;
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

    // rebuild faces
    assert_eq!(map.fetch_faces().identifiers.len(), N_SQUARE.pow(2) * 2);

    black_box(&mut map);
}

fn split_some(mut map: CMap2<FloatType>, split: &[bool]) {
    let n_split = split.len();
    map.fetch_faces()
        .identifiers
        .iter()
        .copied()
        .filter(|square| split[*square as usize % n_split])
        .for_each(|square| {
            let square = square as DartIdentifier;
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

    black_box(map.fetch_faces());
    black_box(&mut map);
}

fn split_diff(mut map: CMap2<FloatType>, split: &[bool]) {
    let n_split = split.len();
    map.fetch_faces()
        .identifiers
        .iter()
        .copied()
        .for_each(|square| {
            let square = square as DartIdentifier;
            let (ddown, dright, dup, dleft) = (square, square + 1, square + 2, square + 3);
            // in a parallel impl, we would create all new darts before-hand
            let dsplit1 = map.add_free_darts(2);
            let dsplit2 = dsplit1 + 1;

            let (dbefore1, dbefore2, dafter1, dafter2) = if split[square as usize % n_split] {
                (ddown, dup, dleft, dright)
            } else {
                (dright, dleft, ddown, dup)
            };

            // unsew the square & duplicate vertices to avoid data loss
            // this duplication effectively means that there are two existing vertices
            // for a short time, before being merged back by the sewing ops
            map.one_unsew(dbefore1);
            map.one_unsew(dbefore2);
            // link the two new dart in order to
            map.two_link(dsplit1, dsplit2);
            // define beta1 of the new darts, i.e. tell them where they point to
            map.one_sew(dsplit1, dafter1);
            map.one_sew(dsplit2, dafter2);

            // sew the original darts to the new darts
            map.one_sew(dbefore1, dsplit1);
            map.one_sew(dbefore2, dsplit2);
            // fuse the edges; this is where duplicated vertices are merged back together
        });

    // rebuild faces
    assert_eq!(map.fetch_faces().identifiers.len(), N_SQUARE.pow(2) * 2);

    black_box(&mut map);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let map: CMap2<FloatType> = GridBuilder::unit_squares(N_SQUARE).build2().unwrap();
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
