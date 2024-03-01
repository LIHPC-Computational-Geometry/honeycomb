//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of key reading methods, classfied into
//! three groups
//!
//! - `bench_read_beta`: benches accesses to beta methods.
//! - `bench_is_free`: benches `is_free` & `is_i_free` methods.
//! - `bench_cell_computation`: benches the `i_cell` method.
//!
//! Each benchmark is repeated on TwoMap of different sizes.
//!

// ------ IMPORTS

use honeycomb_core::{DartIdentifier, TwoMap};
use honeycomb_utils::generation::square_two_map;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

// ------ CONTENT

fn get_map(n_square: usize) -> TwoMap<1> {
    square_two_map::<1>(n_square)
}

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn single_beta_single_dart(map: &TwoMap<1>) -> DartIdentifier {
    black_box(map.beta::<1>(5))
}

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn all_betas_single_dart(map: &TwoMap<1>) -> (DartIdentifier, DartIdentifier, DartIdentifier) {
    (
        black_box(map.beta::<0>(5)),
        black_box(map.beta::<1>(5)),
        black_box(map.beta::<2>(5)),
    )
}

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn single_beta_contiguous_darts(
    map: &TwoMap<1>,
) -> (DartIdentifier, DartIdentifier, DartIdentifier) {
    (
        black_box(map.beta::<1>(5)),
        black_box(map.beta::<1>(6)),
        black_box(map.beta::<1>(7)),
    )
}

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn single_beta_random_darts(map: &TwoMap<1>) -> (DartIdentifier, DartIdentifier, DartIdentifier) {
    (
        black_box(map.beta::<0>(3)),
        black_box(map.beta::<1>(10)),
        black_box(map.beta::<2>(14)),
    )
}

library_benchmark_group!(
    name = bench_read_beta;
    benchmarks =
        single_beta_single_dart,
        all_betas_single_dart,
        single_beta_contiguous_darts,
        single_beta_random_darts
);

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn i_free(map: &TwoMap<1>) -> bool {
    black_box(map.is_i_free::<1>(3))
}

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn free(map: &TwoMap<1>) -> bool {
    black_box(map.is_free(3))
}

library_benchmark_group!(
    name = bench_is_free;
    benchmarks =
        i_free,
        free,
);

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn zero_cell(map: &TwoMap<1>) -> Vec<DartIdentifier> {
    black_box(map.i_cell::<0>(5))
}

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn one_cell(map: &TwoMap<1>) -> Vec<DartIdentifier> {
    black_box(map.i_cell::<0>(5))
}

#[library_benchmark]
#[bench::small(&get_map(5))]
#[bench::medium(&get_map(50))]
#[bench::large(&get_map(500))]
fn two_cell(map: &TwoMap<1>) -> Vec<DartIdentifier> {
    black_box(map.i_cell::<2>(5))
}

library_benchmark_group!(
    name = bench_cell_computation;
    benchmarks =
        zero_cell,
        one_cell,
        two_cell,
);

main!(
    library_benchmark_groups = bench_read_beta,
    bench_is_free,
    bench_cell_computation
);
