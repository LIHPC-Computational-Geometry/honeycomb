use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{
    distributions::{Bernoulli, Distribution},
    rngs::SmallRng,
    SeedableRng,
};

use honeycomb_core::TwoMap;
use honeycomb_utils::generation::square_two_map;

fn split<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>) {
    black_box(&mut map);
}

fn split_some<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>, split: &[bool]) {
    black_box(&mut map);
}

fn split_diff<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>, split: &[bool]) {
    black_box(&mut map);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const N_SQUARE: usize = 2_usize.pow(11);
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
