use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use honeycomb::prelude::{CMap2, CMapBuilder};

use honeycomb_benches::utils::FloatType;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder-grid-size");

    for multiplier in 7..13 {
        let size = 2_u64.pow(multiplier);
        group.throughput(Throughput::Elements(size.pow(2))); // throughoutput = number of cells
        group.bench_with_input(BenchmarkId::new("unit-squares", ""), &size, |b, size| {
            b.iter(|| {
                let mut map: CMap2<FloatType> =
                    CMapBuilder::unit_grid(*size as usize).build().unwrap();
                black_box(&mut map);
            })
        });
        group.bench_with_input(BenchmarkId::new("unit-triangles", ""), &size, |b, size| {
            b.iter(|| {
                let mut map: CMap2<FloatType> =
                    CMapBuilder::unit_triangles(*size as usize).build().unwrap();
                black_box(&mut map);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
