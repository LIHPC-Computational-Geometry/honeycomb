//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of basic operations, classified into three groups:
//!
//! - `bench_darts`: benches dart-related methods.
//! - `bench_vertices`: benches vertex-related methods.
//!
//! Each benchmark is repeated on CMap2 of different sizes.

// ------ IMPORTS

use honeycomb_benches::FloatType;
use honeycomb_core::utils::GridBuilder;
use honeycomb_core::{CMap2, DartIdentifier, Vertex2};
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig,
};
use std::hint::black_box;

// ------ CONTENT

// --- common

fn get_map(n_square: usize) -> CMap2<FloatType> {
    GridBuilder::unit_squares(n_square).build2().unwrap()
}

fn get_sparse_map(n_square: usize) -> CMap2<FloatType> {
    let mut map = GridBuilder::unit_squares(n_square).build2().unwrap();
    map.set_betas(5, [0; 3]); // free dart 5
    map.remove_free_dart(5);
    // because of the way we built the map in the square_cmap2 function & the ID computation
    // policy, we can safely remove a vertex we know is defined
    assert_eq!(map.remove_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    map
}

// --- leftovers
#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn add_single_dart(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.add_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn add_ten_darts(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.add_free_darts(10))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn insert_dart(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.insert_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn insert_dart_full(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.insert_free_dart())
}

library_benchmark_group!(
    name = bench_darts;
    benchmarks =
        insert_dart,
        insert_dart_full,
);

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn insert_vertex(map: &mut CMap2<FloatType>) {
    black_box(map.insert_vertex(1, (0.0, 0.0)));
}

library_benchmark_group!(
    name = bench_vertices;
    benchmarks =
        insert_vertex,
);

// --- main

main!(
    config = LibraryBenchmarkConfig::default().flamegraph(FlamegraphConfig::default());
    library_benchmark_groups = bench_darts,
    bench_vertices,
);
