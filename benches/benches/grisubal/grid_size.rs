// ------ IMPORTS

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use honeycomb::prelude::{
    grisubal::{grisubal, Clip},
    CMap2,
};
use honeycomb_benches::FloatType;

// ------ CONTENT

pub fn criterion_benchmark(c: &mut Criterion) {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect(
        "E: No input geometry specified - you can pass a path to a vtk input as command line argument",
    );

    let mut group = c.benchmark_group("grisubal-grid-size");

    let base_size = 0.1;

    for multiplier in 1..11 {
        let size = base_size * multiplier as FloatType;
        //group.throughput(Throughput::Elements(?));
        group.bench_with_input(BenchmarkId::new("no-clip", ""), &size, |b, size| {
            b.iter(|| {
                let mut map: CMap2<FloatType> = grisubal(path, [*size, *size], Clip::None).unwrap();
                black_box(&mut map);
            })
        });
        group.bench_with_input(BenchmarkId::new("clip-left", ""), &size, |b, size| {
            b.iter(|| {
                let mut map: CMap2<FloatType> = grisubal(path, [*size, *size], Clip::Left).unwrap();
                black_box(&mut map);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
