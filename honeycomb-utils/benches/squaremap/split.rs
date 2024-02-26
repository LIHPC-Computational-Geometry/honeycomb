use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{
    distributions::{Bernoulli, Distribution},
    rngs::SmallRng,
    SeedableRng,
};

use honeycomb_core::{DartIdentifier, SewPolicy, TwoMap, UnsewPolicy};
use honeycomb_utils::generation::square_two_map;

const N_SQUARE: usize = 2_usize.pow(10);

fn split<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>) {
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
    // assert_eq!(map.build_all_faces(), N_SQUARE.pow(2) * 2);

    black_box(&mut map);
}

fn split_some<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>, split: &[bool]) {
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
    // assert_eq!(map.build_all_faces(), N_SQUARE.pow(2) * 2);

    black_box(&mut map);
}

fn split_diff<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>, split: &[bool]) {
    (0..N_SQUARE.pow(2)).for_each(|square| {
        let d1 = (1 + square * 4) as DartIdentifier;
        let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
        // in a parallel impl, we would create all new darts before-hand
        let dsplit1 = map.add_free_darts(2);
        let dsplit2 = dsplit1 + 1;
        map.two_sew(dsplit1, dsplit2, SewPolicy::StretchLeft);
        if split[square] {
            map.one_unsew(d1, UnsewPolicy::DoNothing);
            map.one_unsew(d3, UnsewPolicy::DoNothing);
            map.one_sew(d1, dsplit1, SewPolicy::StretchLeft);
            map.one_sew(d3, dsplit2, SewPolicy::StretchLeft);
            map.one_sew(dsplit1, d4, SewPolicy::StretchRight);
            map.one_sew(dsplit2, d2, SewPolicy::StretchRight);
        } else {
            map.one_unsew(d2, UnsewPolicy::DoNothing);
            map.one_unsew(d4, UnsewPolicy::DoNothing);
            map.one_sew(d2, dsplit1, SewPolicy::StretchLeft);
            map.one_sew(d4, dsplit2, SewPolicy::StretchLeft);
            map.one_sew(dsplit1, d1, SewPolicy::StretchRight);
            map.one_sew(dsplit2, d3, SewPolicy::StretchRight);
        }
    });

    // rebuild faces
    // assert_eq!(map.build_all_faces(), N_SQUARE.pow(2) * 2);

    black_box(&mut map);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let map: TwoMap<1> = square_two_map(N_SQUARE);
    let seed: u64 = 9817498146784;
    let rng = SmallRng::seed_from_u64(seed);
    let dist = Bernoulli::new(0.6).unwrap();
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
