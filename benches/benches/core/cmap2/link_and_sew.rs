//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of key editing methods and constructors,
//! classified into three groups
//!
//! - `bench_links`: benches all variants of the `link` and `unlink` methods.
//! - `bench_sews`: benches all variants of the `sew` and `unsew` methods.
//!
//! Each benchmark is repeated on CMap2 of different sizes.

use std::hint::black_box;

use honeycomb::core::cmap::{CMap2, CMapBuilder};
use iai_callgrind::{
    Callgrind, FlamegraphConfig, LibraryBenchmarkConfig, library_benchmark,
    library_benchmark_group, main,
};

use honeycomb_benches::utils::FloatType;

// --- common

fn get_map(n_square: usize) -> CMap2<FloatType> {
    CMapBuilder::<2, FloatType>::unit_grid(n_square)
        .build()
        .unwrap()
}

fn get_link_map(n_square: usize) -> CMap2<FloatType> {
    CMapBuilder::<2, FloatType>::from_n_darts(n_square.pow(2) * 4)
        .build()
        .unwrap()
}

fn get_sew_map(n_square: usize) -> CMap2<FloatType> {
    let map = CMapBuilder::<2, FloatType>::from_n_darts(n_square.pow(2) * 4)
        .build()
        .unwrap();
    map.force_write_vertex(4, (0.0, 0.0));
    map.force_write_vertex(6, (1.0, 0.0));
    map
}

// --- link group

#[library_benchmark]
#[bench::small(&mut get_link_map(16))]
#[bench::medium(&mut get_link_map(64))]
#[bench::large(&mut get_link_map(256))]
fn one_link(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_link::<1>(4, 6).unwrap();
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_link_map(16))]
#[bench::medium(&mut get_link_map(64))]
#[bench::large(&mut get_link_map(256))]
fn two_link(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_link::<2>(4, 6).unwrap();
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn one_unlink(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_unlink::<1>(4).unwrap();
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn two_unlink(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_unlink::<2>(4).unwrap();
    black_box(map)
}

library_benchmark_group!(
    name = bench_links;
    benchmarks =
        one_link,
        two_link,
        one_unlink,
        two_unlink,
);

// --- sew group

#[library_benchmark]
#[bench::small(&mut get_sew_map(16))]
#[bench::medium(&mut get_sew_map(64))]
#[bench::large(&mut get_sew_map(256))]
fn one_sew(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_sew::<1>(4, 6).unwrap();
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_sew_map(16))]
#[bench::medium(&mut get_sew_map(64))]
#[bench::large(&mut get_sew_map(256))]
fn two_sew(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_sew::<2>(4, 6).unwrap();
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn one_unsew(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_unsew::<1>(4).unwrap();
    black_box(map)
}

#[library_benchmark]
#[bench::small(&mut get_map(16))]
#[bench::medium(&mut get_map(64))]
#[bench::large(&mut get_map(256))]
fn two_unsew(map: &mut CMap2<FloatType>) -> &mut CMap2<FloatType> {
    map.force_unsew::<2>(4).unwrap();
    black_box(map)
}

library_benchmark_group!(
    name = bench_sews;
    benchmarks =
        one_sew,
        two_sew,
        one_unsew,
        two_unsew,
);

// --- main

main!(
    config = LibraryBenchmarkConfig::default().tool(
        Callgrind::default().flamegraph(FlamegraphConfig::default())
    );
    library_benchmark_groups =
        bench_links,
        bench_sews,
);
