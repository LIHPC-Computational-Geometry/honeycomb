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

use std::hint::black_box;

use honeycomb::{
    core::cmap::DartReservationError,
    prelude::{CMap2, CMapBuilder, DartIdType, Vertex2, grid_generation::GridBuilder},
};
use iai_callgrind::{
    Callgrind, FlamegraphConfig, LibraryBenchmarkConfig, library_benchmark,
    library_benchmark_group, main,
};

use honeycomb_benches::utils::FloatType;

// --- common

fn get_map(n_square: usize) -> CMap2<FloatType> {
    GridBuilder::<2, FloatType>::unit_grid(n_square)
}

fn get_sparse_map(n_square: usize) -> CMap2<FloatType> {
    let map = GridBuilder::<2, FloatType>::unit_grid(n_square);
    map.set_betas(5, [0; 3]); // free dart 5
    map.release_dart(5).unwrap();
    // because of the way we built the map in the square_cmap2 function & the ID computation
    // policy, we can safely remove a vertex we know is defined
    assert_eq!(
        map.remove_vertex(1).unwrap(),
        Vertex2::from((0.0, 0.0))
    );
    map
}

fn get_empty_map(n_squares: usize) -> (CMap2<FloatType>, usize) {
    (
        CMapBuilder::<2>::from_n_darts(0).build().unwrap(),
        n_squares.pow(2) * 4,
    )
}

// --- dart group

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512], setup = get_empty_map)]
fn add_many_darts((mut map, n_darts): (CMap2<FloatType>, usize)) -> DartIdType {
    black_box(map.allocate_used_darts(n_darts))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn insert_dart(map: &mut CMap2<FloatType>) -> Vec<DartIdType> {
    black_box(map.reserve_darts(1).unwrap())
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn insert_dart_full(map: &mut CMap2<FloatType>) -> DartReservationError {
    black_box(map.reserve_darts(1).unwrap_err())
}

#[library_benchmark]
#[bench::small(&mut CMapBuilder::<2>::from_n_darts(16_usize.pow(2) * 4).build().unwrap())]
#[bench::medium(&mut CMapBuilder::<2>::from_n_darts(64_usize.pow(2) * 4).build().unwrap())]
#[bench::large(&mut CMapBuilder::<2>::from_n_darts(256_usize.pow(2) * 4).build().unwrap())]
fn remove_dart(map: &mut CMap2<FloatType>) {
    map.release_dart(5).unwrap();
    black_box(map);
}

library_benchmark_group!(
    name = bench_darts;
    benchmarks =
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
    black_box(map.read_vertex(1))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn read_missing_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.read_vertex(1))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn insert_vertex(map: &mut CMap2<FloatType>) {
    map.write_vertex(1, (0.0, 0.0));
    black_box(map);
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn replace_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.write_vertex(1, (0.0, 0.0)))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn set_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.write_vertex(1, (0.0, 0.0)))
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn remove_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.remove_vertex(1))
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(16))]
#[bench::medium(&mut get_sparse_map(64))]
#[bench::large(&mut get_sparse_map(256))]
fn remove_missing_vertex(map: &mut CMap2<FloatType>) -> Option<Vertex2<FloatType>> {
    black_box(map.remove_vertex(1))
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
    config = LibraryBenchmarkConfig::default().tool(
        Callgrind::default().flamegraph(FlamegraphConfig::default())
    );
    library_benchmark_groups = bench_darts,
    bench_vertices,
);
