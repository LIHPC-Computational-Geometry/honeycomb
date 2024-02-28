use honeycomb_core::{DartIdentifier, TwoMap};
use honeycomb_utils::generation::square_two_map;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

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

main!(library_benchmark_groups = bench_read_beta);
