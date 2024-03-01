//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of key editing methods and constructors,
//! classfied into three groups
//!
//! - `bench_one_sewing`: benches the `one_sew` method over all sewing policies.
//! - `bench_two_sewing`: benches the `two_sew` method over all sewing policies.
//! - `bench_one_unsewing`: benches the `one_unsew` method over all unsewing policies.
//! - `bench_two_unsewing`: benches the `two_unsew` method over all unsewing policies.
//!
//! Each benchmark is repeated on TwoMap of different sizes.
//!

// ------ IMPORTS

use honeycomb_core::{SewPolicy, TwoMap, UnsewPolicy};
use honeycomb_utils::generation::square_two_map;
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig,
};
use std::hint::black_box;

// ------ CONTENT

fn compute_dims(n_square: usize) -> (usize, usize) {
    (n_square.pow(2) * 4, (n_square + 1).pow(2))
}

fn get_map(n_square: usize) -> TwoMap<1> {
    square_two_map::<1>(n_square)
}

fn get_unstructured_map(n_square: usize) -> TwoMap<1> {
    let (n_darts, n_vertices) = compute_dims(n_square);
    TwoMap::new(n_darts, n_vertices)
}

#[library_benchmark]
#[bench::small(&mut get_unstructured_map(5))]
#[bench::medium(&mut get_unstructured_map(50))]
#[bench::large(&mut get_unstructured_map(500))]
fn one_sew_left(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.one_sew(4, 6, SewPolicy::StretchLeft);
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_unstructured_map(5))]
#[bench::medium(&mut get_unstructured_map(50))]
#[bench::large(&mut get_unstructured_map(500))]
fn one_sew_right(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.one_sew(4, 6, SewPolicy::StretchRight);
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_unstructured_map(5))]
#[bench::medium(&mut get_unstructured_map(50))]
#[bench::large(&mut get_unstructured_map(500))]
fn one_sew_avg(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.one_sew(4, 6, SewPolicy::StretchAverage);
    black_box(map)
}

library_benchmark_group!(
    name = bench_one_sewing;
    benchmarks =
        one_sew_left,
        one_sew_right,
        one_sew_avg,
);

#[library_benchmark]
#[bench::small(&mut get_unstructured_map(5))]
#[bench::medium(&mut get_unstructured_map(50))]
#[bench::large(&mut get_unstructured_map(500))]
fn two_sew_left(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.two_sew(4, 6, SewPolicy::StretchLeft);
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_unstructured_map(5))]
#[bench::medium(&mut get_unstructured_map(50))]
#[bench::large(&mut get_unstructured_map(500))]
fn two_sew_right(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.two_sew(4, 6, SewPolicy::StretchRight);
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_unstructured_map(5))]
#[bench::medium(&mut get_unstructured_map(50))]
#[bench::large(&mut get_unstructured_map(500))]
fn two_sew_avg(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.two_sew(4, 6, SewPolicy::StretchAverage);
    black_box(map)
}

library_benchmark_group!(
    name = bench_two_sewing;
    benchmarks =
        two_sew_left,
        two_sew_right,
        two_sew_avg,
);

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn one_unsew_nothing(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.one_unsew(4, UnsewPolicy::DoNothing);
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn one_unsew_duplicate(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.one_unsew(4, UnsewPolicy::Duplicate);
    black_box(map)
}

library_benchmark_group!(
    name = bench_one_unsewing;
    benchmarks =
        one_unsew_nothing,
        one_unsew_duplicate,
);

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn two_unsew_nothing(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.two_unsew(4, UnsewPolicy::DoNothing);
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn two_unsew_duplicate(map: &mut TwoMap<1>) -> &mut TwoMap<1> {
    map.two_unsew(4, UnsewPolicy::Duplicate);
    black_box(map)
}

library_benchmark_group!(
    name = bench_two_unsewing;
    benchmarks =
        two_unsew_nothing,
        two_unsew_duplicate,
);

main!(
    config = LibraryBenchmarkConfig::default().flamegraph(FlamegraphConfig::default());
    library_benchmark_groups = bench_one_sewing,
    bench_two_sewing,
    bench_one_unsewing,
    bench_two_unsewing
);
