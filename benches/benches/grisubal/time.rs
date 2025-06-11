use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use honeycomb::prelude::{
    CMap2,
    grisubal::{Clip, grisubal},
};

use honeycomb_benches::utils::FloatType;

pub fn criterion_benchmark(c: &mut Criterion) {
    // passing args to cargo bench filters bench instead of actually reading args;
    // the path to the file needs to be hardcoded (I think?)
    //let path = "./examples/shape.vtk";
    let path = "../examples/shape.vtk";

    let mut group = c.benchmark_group("grisubal-run-time");

    let size = 0.1;

    group.bench_with_input(
        BenchmarkId::new("no-clip", ""),
        &(path, size),
        |b, (path, size)| {
            b.iter(|| {
                let mut map: CMap2<FloatType> = grisubal(path, [*size, *size], Clip::None).unwrap();
                black_box(&mut map);
            })
        },
    );
    group.bench_with_input(BenchmarkId::new("clip-left", ""), &size, |b, size| {
        b.iter(|| {
            let mut map: CMap2<FloatType> = grisubal(path, [*size, *size], Clip::Left).unwrap();
            black_box(&mut map);
        })
    });
    group.bench_with_input(BenchmarkId::new("clip-right", ""), &size, |b, size| {
        b.iter(|| {
            let mut map: CMap2<FloatType> = grisubal(path, [*size, *size], Clip::Right).unwrap();
            black_box(&mut map);
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
