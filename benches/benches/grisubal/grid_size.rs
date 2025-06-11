use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use honeycomb::prelude::{
    CMap2,
    grisubal::{Clip, grisubal},
};

use honeycomb_benches::utils::FloatType;

pub fn criterion_benchmark(c: &mut Criterion) {
    // passing args to cargo bench filters bench instead of actually reading args;
    // the path to the file needs to be hardcoded (I think?)
    let path = "../examples/shape.vtk";

    let mut group = c.benchmark_group("grisubal-grid-size");

    let base_size = 0.1;

    for multiplier in 1..11 {
        let size = base_size * multiplier as FloatType;
        group.throughput(Throughput::Elements((1. / size.powi(2)) as u64));
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
