//! This benchmarks handle measurements of the initialization speed
//! of the CMap2 structure in the context of an orthogonal 2D mesh
//! representation, each cell being split diagonally. The exact
//! structure is described in the doc of the `generation::square_two_map method`.

// ------ IMPORTS

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};
use honeycomb_core::{CMap2, FloatType};
use honeycomb_utils::generation::splitsquare_cmap2;

// ------ CONTENT

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("splitsquaremap-init");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    for pow in 5..13 {
        let n_square = 2_usize.pow(pow);
        group.throughput(Throughput::Elements(n_square.pow(2) as u64));
        group.bench_with_input(BenchmarkId::new("init", ""), &n_square, |b, n_square| {
            b.iter(|| {
                let mut map: CMap2<FloatType> = splitsquare_cmap2(*n_square);
                black_box(&mut map);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
