//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of key editing methods and constructors,
//! classfied into three groups:
//!
//! - `bench_new`: benches constructors and "add" methods.
//! - `bench_insert`: benches insertion methods (both behaviors).
//! - `bench_face_building`: benches the face building methods.
//!
//! Each benchmark is repeated on TwoMap of different sizes.
//!

// ------ IMPORTS

use honeycomb_core::{DartIdentifier, FaceIdentifier, TwoMap, VertexIdentifier};
use honeycomb_utils::generation::square_two_map;
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig,
};
use std::hint::black_box;

// ------ CONTENT

fn get_map(n_square: usize) -> TwoMap<1> {
    square_two_map::<1>(n_square)
}

fn get_sparse_map(n_square: usize) -> TwoMap<1> {
    let mut map = square_two_map::<1>(n_square);
    map.set_betas(5, [0; 3]); // free dart 5
    map.remove_free_dart(5);
    map.remove_vertex(1);
    map
}

fn compute_dims(n_square: usize) -> (usize, usize) {
    (n_square.pow(2) * 4, (n_square + 1).pow(2))
}

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512], setup = compute_dims)]
fn constructor((n_darts, n_vertices): (usize, usize)) -> TwoMap<1> {
    black_box(TwoMap::new(n_darts, n_vertices))
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn add_single_dart(map: &mut TwoMap<1>) -> DartIdentifier {
    black_box(map.add_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn add_ten_darts(map: &mut TwoMap<1>) -> DartIdentifier {
    black_box(map.add_free_darts(10))
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn add_default_vertex(map: &mut TwoMap<1>) -> VertexIdentifier {
    black_box(map.add_vertex(None))
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn add_vertex(map: &mut TwoMap<1>) -> VertexIdentifier {
    black_box(map.add_vertex(Some([12.0, 6.0])))
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn add_10_vertices(map: &mut TwoMap<1>) -> VertexIdentifier {
    black_box(map.add_vertices(10))
}

library_benchmark_group!(
    name = bench_new;
    benchmarks =
        constructor,
        add_single_dart,
        add_ten_darts,
        add_default_vertex,
        add_vertex,
        add_10_vertices,
);

#[library_benchmark]
#[bench::small(&mut get_sparse_map(5))]
#[bench::medium(&mut get_sparse_map(50))]
#[bench::large(&mut get_sparse_map(500))]
fn insert_dart(map: &mut TwoMap<1>) -> DartIdentifier {
    black_box(map.insert_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn insert_dart_full(map: &mut TwoMap<1>) -> DartIdentifier {
    black_box(map.insert_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(5))]
#[bench::medium(&mut get_sparse_map(50))]
#[bench::large(&mut get_sparse_map(500))]
fn insert_vertex(map: &mut TwoMap<1>) -> VertexIdentifier {
    black_box(map.insert_vertex(None))
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn insert_vertex_full(map: &mut TwoMap<1>) -> VertexIdentifier {
    black_box(map.insert_vertex(None))
}

library_benchmark_group!(
    name = bench_insert;
    benchmarks =
        insert_dart,
        insert_dart_full,
        insert_vertex,
        insert_vertex_full,
);

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn build_face(map: &mut TwoMap<1>) -> FaceIdentifier {
    black_box(map.build_face(5))
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn build_faces(map: &mut TwoMap<1>) -> usize {
    black_box(map.build_all_faces())
}

library_benchmark_group!(
    name = bench_face_building;
    benchmarks =
        build_face,
        build_faces,
);

main!(
    config = LibraryBenchmarkConfig::default().flamegraph(FlamegraphConfig::default());
    library_benchmark_groups = bench_new,
    bench_insert,
    bench_face_building,
);
