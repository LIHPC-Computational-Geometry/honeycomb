use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use honeycomb::prelude::{CMap2, grid_generation::GridBuilder};

use honeycomb_benches::utils::FloatType;

pub fn criterion_benchmark(c: &mut Criterion) {
    // passing args to cargo bench filters bench instead of actually reading args;
    // the path to the file needs to be hardcoded (I think?)
    //let path = "./examples/shape.vtk";

    let mut group = c.benchmark_group("builder-time");

    let n_square = 128;

    group.bench_with_input(BenchmarkId::new("unit-squares", ""), &(), |b, _| {
        b.iter(|| {
            let mut map: CMap2<FloatType> = GridBuilder::<2, _>::unit_grid(n_square);
            black_box(&mut map);
        })
    });
    group.bench_with_input(BenchmarkId::new("unit-triangles", ""), &(), |b, _| {
        b.iter(|| {
            let mut map: CMap2<FloatType> = GridBuilder::<2, _>::unit_triangles(n_square);
            black_box(&mut map);
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
