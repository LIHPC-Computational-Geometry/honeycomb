//! This benchmarks handle measurements for a given operation on CMap2
//! of a given topology (see `generation::splitsquare_two_map` doc).
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
};

use honeycomb_core::{
    utils::GridBuilder, CMap2, DartIdentifier, FloatType, Vector2, VertexIdentifier, NULL_DART_ID,
};

// ------ CONTENT

fn offset(mut map: CMap2<FloatType>, offsets: &[Vector2<FloatType>]) {
    let n_offsets = offsets.len();
    let vertices = map.fetch_vertices();
    vertices.identifiers.iter().for_each(|vertex_id| {
        let current_value = map.vertex(*vertex_id as DartIdentifier).unwrap();
        let _ = map.replace_vertex(
            *vertex_id as VertexIdentifier,
            current_value + offsets[*vertex_id as usize % n_offsets],
        );
    });
    black_box(&mut map);
}

fn offset_if_inner(mut map: CMap2<FloatType>, offsets: &[Vector2<FloatType>]) {
    let n_offsets = offsets.len();
    let mut inner: BTreeSet<VertexIdentifier> = BTreeSet::new();
    let vertices = map.fetch_vertices();
    // collect inner vertex IDs
    vertices.identifiers.iter().for_each(|vertex_id| {
        let neighbors_vertex_cell: Vec<DartIdentifier> = map
            .i_cell::<0>(*vertex_id as DartIdentifier)
            .map(|d_id| map.beta::<2>(d_id))
            .collect();
        if !neighbors_vertex_cell.contains(&NULL_DART_ID) {
            inner.insert(*vertex_id);
        }
    });
    inner.iter().for_each(|vertex_id| {
        let current_value = map.vertex(*vertex_id).unwrap();
        let _ = map.replace_vertex(
            *vertex_id,
            current_value + offsets[*vertex_id as usize % n_offsets],
        );
    });
    black_box(&mut map);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const N_SQUARE: usize = 2_usize.pow(11);
    let map: CMap2<FloatType> = GridBuilder::split_unit_squares(N_SQUARE).build2().unwrap();
    let seed: u64 = 9817498146784;
    let mut rngx = SmallRng::seed_from_u64(seed);
    let mut rngy = SmallRng::seed_from_u64(seed);
    let range: Uniform<FloatType> =
        Uniform::new(-0.5, 0.5).expect("Could not initialize the uniform distribution");
    let xs = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngx));
    let ys = (0..(N_SQUARE + 1).pow(2)).map(|_| range.sample(&mut rngy));

    let offsets: Vec<Vector2<FloatType>> = xs.zip(ys).map(|(x, y)| (x, y).into()).collect();

    let mut group = c.benchmark_group("splitsquaremap-shift");

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
