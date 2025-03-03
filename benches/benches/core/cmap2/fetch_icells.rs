use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use honeycomb::core::cmap::{CMap2, CMapBuilder};

use honeycomb_benches::utils::FloatType;

pub fn criterion_benchmark(c: &mut Criterion) {
    let n_square = 512;
    let map: CMap2<FloatType> = CMapBuilder::unit_grid(n_square).build().unwrap();

    let mut group = c.benchmark_group("fetch-icells");

    group.bench_with_input(BenchmarkId::new("fetch-vertices", ""), &map, |b, m| {
        b.iter(|| {
            let mut vertices: Vec<_> = m.iter_vertices().collect();
            black_box(&mut vertices);
        })
    });
    group.bench_with_input(BenchmarkId::new("fetch-edges", ""), &map, |b, m| {
        b.iter(|| {
            let mut edges: Vec<_> = m.iter_edges().collect();
            black_box(&mut edges);
        })
    });
    group.bench_with_input(BenchmarkId::new("fetch-faces", ""), &map, |b, m| {
        b.iter(|| {
            let mut faces: Vec<_> = m.iter_faces().collect();
            black_box(&mut faces);
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
