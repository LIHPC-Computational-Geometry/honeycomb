use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    SeedableRng,
};

use honeycomb_core::{TwoMap, Vertex2};
use honeycomb_utils::generation::square_two_map;

fn offset<const N_MARKS: usize>(map: TwoMap<N_MARKS>, offsets: &[Vertex2]) {
    todo!()
}

fn offset_if_inner<const N_MARKS: usize>(map: &mut TwoMap<N_MARKS>, offsets: &[Vertex2]) {
    todo!()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const N_SQUARE: usize = 2_usize.pow(11);
    let mut map: TwoMap<1> = square_two_map(N_SQUARE);
    let seed: u64 = 9817498146784;
    let mut rngx = SmallRng::seed_from_u64(seed);
    let mut rngy = SmallRng::seed_from_u64(seed);
    let range: Uniform<f64> = rand::distributions::Uniform::new(-0.5, 0.5);
    let xs = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngx));
    let ys = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngy));

    let offsets: Vec<Vertex2> = xs.zip(ys).map(|(x, y)| [x, y]).collect();

    let mut group = c.benchmark_group("squaremap-shift");
    /*
    group.bench_with_input(
        BenchmarkId::new("precomputed-offsets", ""),
        &(map, offsets),
        |b, (map, offsets)| b.iter(|| offset(*map, &offsets)),
    );
    */
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
