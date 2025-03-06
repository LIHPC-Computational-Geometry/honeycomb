//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of key building methods and constructors,
//! classified into two groups:
//!
//! - `bench_constructors`: benches constructors functions.
//! - `bench_fetches`: benches insertion methods (both behaviors).
//! - `bench_icells`: benches the i-cell method
//!
//! Each benchmark is repeated on CMap2 of different sizes.

use std::hint::black_box;

use honeycomb::core::cmap::{CMap2, CMapBuilder};
use iai_callgrind::{
    FlamegraphConfig, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main,
};

use honeycomb_benches::utils::FloatType;

// --- common

fn get_map(n_square: usize) -> CMap2<FloatType> {
    CMapBuilder::unit_grid(n_square).build().unwrap()
}

// --- constructor group

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512])]
fn new(n_squares: usize) -> CMap2<FloatType> {
    black_box(
        CMapBuilder::from_n_darts(n_squares.pow(2) * 4)
            .build()
            .unwrap(),
    )
}

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512])]
fn grid(n_squares: usize) -> CMap2<FloatType> {
    black_box(CMapBuilder::unit_grid(n_squares).build().unwrap())
}

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512])]
fn tet_grid(n_squares: usize) -> CMap2<FloatType> {
    black_box(CMapBuilder::unit_triangles(n_squares).build().unwrap())
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
fn iter_vertices(map: &mut CMap2<FloatType>) {
    black_box(map.iter_vertices().collect::<Vec<_>>());
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn iter_edges(map: &mut CMap2<FloatType>) {
    black_box(map.iter_edges().collect::<Vec<_>>());
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn iter_faces(map: &mut CMap2<FloatType>) {
    black_box(map.iter_faces().collect::<Vec<_>>());
}

library_benchmark_group!(
    name = bench_fetches;
    benchmarks =
        iter_vertices,
        iter_edges,
        iter_faces,
);

// --- i-cell group

#[library_benchmark]
#[bench::small(&get_map(16))]
#[bench::medium(&get_map(64))]
#[bench::large(&get_map(256))]
fn zero_cell(map: &CMap2<FloatType>) {
    black_box(map.i_cell::<0>(5));
}

#[library_benchmark]
#[bench::small(&get_map(16))]
#[bench::medium(&get_map(64))]
#[bench::large(&get_map(256))]
fn one_cell(map: &CMap2<FloatType>) {
    black_box(map.i_cell::<1>(5));
}

#[library_benchmark]
#[bench::small(&get_map(16))]
#[bench::medium(&get_map(64))]
#[bench::large(&get_map(256))]
fn two_cell(map: &CMap2<FloatType>) {
    black_box(map.i_cell::<2>(5));
}

library_benchmark_group!(
    name = bench_icells;
    benchmarks =
        zero_cell,
        one_cell,
        two_cell,
);

// --- main

main!(
    config = LibraryBenchmarkConfig::default().flamegraph(FlamegraphConfig::default());
    library_benchmark_groups =
        bench_constructors,
        bench_fetches,
        bench_icells,
);
