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

use std::hint::black_box;

use honeycomb_core::cmap::{CMap2, CMapBuilder, DartIdType};
use honeycomb_core::geometry::Vertex2;
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig,
};

use honeycomb_benches::FloatType;

// ------ CONTENT

// --- common

fn get_map(n_square: usize) -> CMap2<FloatType> {
    CMapBuilder::unit_grid(n_square).build().unwrap()
}

fn get_sparse_map(n_square: usize) -> CMap2<FloatType> {
    let mut map = CMapBuilder::unit_grid(n_square).build().unwrap();
    map.set_betas(5, [0; 3]); // free dart 5
    map.remove_free_dart(5);
    // because of the way we built the map in the square_cmap2 function & the ID computation
    // policy, we can safely remove a vertex we know is defined
    assert_eq!(
        map.force_remove_vertex(1).unwrap(),
        Vertex2::from((0.0, 0.0))
    );
    map
}

fn get_empty_map(n_squares: usize) -> (CMap2<FloatType>, usize) {
    (
        CMapBuilder::default().build().unwrap(),
        n_squares.pow(2) * 4,
    )
}

// --- dart group

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn add_single_dart(map: &mut CMap2<FloatType>) -> DartIdType {
    black_box(map.add_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn add_ten_darts(map: &mut CMap2<FloatType>) -> DartIdType {
    black_box(map.add_free_darts(10))
}

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512], setup = get_empty_map)]
fn add_many_darts((mut map, n_darts): (CMap2<FloatType>, usize)) -> DartIdType {
    black_box(map.add_free_darts(n_darts))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn insert_dart(map: &mut CMap2<FloatType>) -> DartIdType {
    black_box(map.insert_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn insert_dart_full(map: &mut CMap2<FloatType>) -> DartIdType {
    black_box(map.insert_free_dart())
}

#[library_benchmark]
#[bench::small(&mut CMapBuilder::default().n_darts(16_usize.pow(2) * 4).build().unwrap())]
#[bench::medium(&mut CMapBuilder::default().n_darts(64_usize.pow(2) * 4).build().unwrap())]
#[bench::large(&mut CMapBuilder::default().n_darts(256_usize.pow(2) * 4).build().unwrap())]
fn remove_dart(map: &mut CMap2<FloatType>) {
    map.remove_free_dart(5);
    black_box(map);
}

library_benchmark_group!(
    name = bench_darts;
    benchmarks =
        add_single_dart,
        add_ten_darts,
        add_many_darts,
        insert_dart,
        insert_dart_full,
);

// --- vertex group

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn read_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.force_read_vertex(1))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn read_missing_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.force_read_vertex(1))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn insert_vertex(map: &mut CMap2<FloatType>) {
    map.force_write_vertex(1, (0.0, 0.0));
    black_box(map);
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn replace_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.force_write_vertex(1, (0.0, 0.0)))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn set_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.force_write_vertex(1, (0.0, 0.0)))
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn remove_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.force_remove_vertex(1))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn remove_missing_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.force_remove_vertex(1))
}

library_benchmark_group!(
    name = bench_vertices;
    benchmarks =
        read_vertex,
        read_missing_vertex,
        insert_vertex,
        replace_vertex,
        set_vertex,
        remove_vertex,
        remove_missing_vertex,
);

// --- main

main!(
    config = LibraryBenchmarkConfig::default().flamegraph(FlamegraphConfig::default());
    library_benchmark_groups = bench_darts,
    bench_vertices,
);
