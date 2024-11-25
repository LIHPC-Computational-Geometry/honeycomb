// ------ IMPORTS

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use honeycomb::{kernels::shift::shift_vertices_to_neigh_avg, prelude::CMap2};
use honeycomb_benches::FloatType;
use honeycomb_core::cmap::CMapBuilder;

// ------ CONTENT

pub fn criterion_benchmark(c: &mut Criterion) {
    let path = "../../../meshing-samples/vtk/2D/many_quads.vtk";

    let mut group = c.benchmark_group("shift");

    let mut map: CMap2<FloatType> = CMapBuilder::from(path).build().unwrap();

    group.bench_function(BenchmarkId::new("to-neighbor-avg", ""), |b| {
        b.iter(|| {
            shift_vertices_to_neigh_avg(&map, 200);
            black_box(&mut map);
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
