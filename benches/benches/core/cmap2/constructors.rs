//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of key building methods and constructors,
//! classified into two groups:
//!
//! - `bench_constructors`: benches constructors functions.
//! - `bench_fetches`: benches insertion methods (both behaviors).
//!
//! Each benchmark is repeated on CMap2 of different sizes.

// ------ IMPORTS

use honeycomb_benches::FloatType;
use honeycomb_core::{utils::GridBuilder, CMap2, DartIdentifier, Vertex2};
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig,
};
use std::hint::black_box;

// ------ CONTENT

// --- common

fn get_map(n_square: usize) -> CMap2<FloatType> {
    GridBuilder::unit_squares(n_square).build2().unwrap()
}

// --- constructor group

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512])]
fn new(n_squares: usize) -> CMap2<FloatType> {
    black_box(CMap2::new(n_squares.pow(2) * 4))
}

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512])]
fn grid(n_squares: usize) -> CMap2<FloatType> {
    black_box(GridBuilder::unit_squares(n_squares).build2().unwrap())
}

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512])]
fn tet_grid(n_squares: usize) -> CMap2<FloatType> {
    black_box(GridBuilder::split_unit_squares(n_squares).build2().unwrap())
}

library_benchmark_group!(
    name = bench_constructors;
    benchmarks =
        new,
        grid,
        tet_grid,
);

// --- cell fetch group

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn fetch_vertices(map: &mut CMap2<FloatType>) {
    black_box(map.fetch_vertices());
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn fetch_edges(map: &mut CMap2<FloatType>) {
    black_box(map.fetch_edges());
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn fetch_faces(map: &mut CMap2<FloatType>) {
    black_box(map.fetch_faces());
}

library_benchmark_group!(
    name = bench_fetches;
    benchmarks =
        fetch_vertices,
        fetch_edges,
        fetch_faces,
);

// --- main

main!(
    config = LibraryBenchmarkConfig::default().flamegraph(FlamegraphConfig::default());
    library_benchmark_groups = bench_constructors,
    bench_fetches,
);
