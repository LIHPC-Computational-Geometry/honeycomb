// ------ IMPORTS

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use honeycomb::prelude::CMap2;
use honeycomb_benches::FloatType;
use honeycomb_core::cmap::CMapBuilder;

// ------ CONTENT

pub fn criterion_benchmark(c: &mut Criterion) {
    let n_square = 512;
    let map: CMap2<FloatType> = CMapBuilder::unit_grid(n_square).build().unwrap();

    let mut group = c.benchmark_group("fetch-icells");

    group.bench_with_input(BenchmarkId::new("fetch-vertices", ""), &map, |b, m| {
        b.iter(|| {
            let mut vertices = m.fetch_vertices();
            black_box(&mut vertices);
        })
    });
    group.bench_with_input(BenchmarkId::new("fetch-edges", ""), &map, |b, m| {
        b.iter(|| {
            let mut edges = m.fetch_edges();
            black_box(&mut edges);
        })
    });
    group.bench_with_input(BenchmarkId::new("fetch-faces", ""), &map, |b, m| {
        b.iter(|| {
            let mut faces = m.fetch_faces();
            black_box(&mut faces);
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);