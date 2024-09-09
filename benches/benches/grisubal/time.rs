// ------ IMPORTS

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use honeycomb::prelude::{
    grisubal::{grisubal, Clip},
    CMap2,
};
use honeycomb_benches::FloatType;

// ------ CONTENT

pub fn criterion_benchmark(c: &mut Criterion) {
    // passing args to cargo bench filters bench instead of actually reading args;
    // the path to the file needs to be hardcoded (I think?)
    // let args: Vec<String> = std::env::args().collect();
    let path = "path/to/file.vtk";
    /*
    args.get(1).expect(
        "E: No input geometry specified - you can pass a path to a vtk input as command line argument",
    );
    */

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
