//! This benchmarks handle measurements for a given operation on TwoMap
//! of a given topology (see `generation::square_two_map` doc).
//!
//! The operations applied here affect only geometry, topology is left unchanged
//!
//! The offset operation consists in shifting the position of all vertices
//! of the map randomly; Each vertex is moved in the range (-0.5, 0.5) from
//! its initial position, along both coordinates.
//!
//! The offset_if_inner operation consists has the same effect, but is only
//! applied to vertices that are not on the border of the map, i.e. the
//! vertices on the border stay at the same position while the inner ones
//! are displaced.

// ------ IMPORTS

use std::collections::BTreeSet;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    SeedableRng,
};

use honeycomb_core::{DartIdentifier, TwoMap, Vertex2, VertexIdentifier, NULL_DART_ID};
use honeycomb_utils::generation::square_two_map;

// ------ CONTENT

fn offset<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>, offsets: &[Vertex2]) {
    (0..map.n_vertices().0).for_each(|vertex_id| {
        let _ = map.set_vertex(vertex_id as VertexIdentifier, offsets[vertex_id]);
    });
    black_box(&mut map);
}

fn offset_if_inner<const N_MARKS: usize>(mut map: TwoMap<N_MARKS>, offsets: &[Vertex2]) {
    let mut inner: BTreeSet<VertexIdentifier> = BTreeSet::new();
    // collect inner vertex IDs
    (0..map.n_darts().0 as DartIdentifier).for_each(|dart_id| {
        let neighbors_vertex_cell: Vec<DartIdentifier> = map
            .i_cell::<0>(dart_id)
            .iter()
            .map(|d_id| map.beta::<2>(*d_id))
            .collect();
        if !neighbors_vertex_cell.contains(&NULL_DART_ID) {
            inner.insert(map.vertexid(dart_id));
        }
    });
    inner.iter().for_each(|vertex_id| {
        let current_value = map.vertex(*vertex_id);
        let _ = map.set_vertex(*vertex_id, *current_value + offsets[*vertex_id as usize]);
    });
    black_box(&mut map);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const N_SQUARE: usize = 2_usize.pow(11);
    let map: TwoMap<1> = square_two_map(N_SQUARE);
    let seed: u64 = 9817498146784;
    let mut rngx = SmallRng::seed_from_u64(seed);
    let mut rngy = SmallRng::seed_from_u64(seed);
    let range: Uniform<f64> = rand::distributions::Uniform::new(-0.5, 0.5);
    let xs = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngx));
    let ys = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngy));

    let offsets: Vec<Vertex2> = xs.zip(ys).map(|(x, y)| (x, y).into()).collect();

    let mut group = c.benchmark_group("squaremap-shift");

    group.bench_with_input(
        BenchmarkId::new("precomputed-offsets", ""),
        &(map.clone(), offsets.clone()),
        |b, (map, offsets)| b.iter(|| offset(map.clone(), offsets)),
    );
    group.bench_with_input(
        BenchmarkId::new("precomputed-offsets-if-inner", ""),
        &(map.clone(), offsets.clone()),
        |b, (map, offsets)| b.iter(|| offset_if_inner(map.clone(), offsets)),
    );

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
