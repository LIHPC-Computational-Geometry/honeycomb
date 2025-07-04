//! # honeycomb-benches
//!
//! This crate contains all benchmarks of the project. It also contains simple binaries used to
//! profile and further optimize the implementation.
//!
//! ## Binary
//!
//! The package provides a single binary, `hc-bench`, which exposes several benchmarks as
//! subcommands. For details on options and arguments, run:
//!
//! ```sh
//! cargo run --bin hc-bench -- --help
//! ```
//!
//! Benchmarks are described in the documentation of their respective modules.
//!
//! ## Features
//!
//! Optional features can be enabled to affect implementation:
//!
//! - `bind-threads` -- enabled by default -- uses `hwlocality` to bind threads to physical cores,
//! - `jemalloc` -- uses `tikv-jemallocator` to replace the default allocator,
//! - `profiling` -- enable `perf` fifo interactions to allow per-section profiling,
//! - `_single_precision` -- compile cargo benches (not the binary) to use `f32` instead of `f64`.
//!
//! ## Available benchmarks
//!
//! ### Criterion-based
//!
//! - `builder` - grid building routines at fixed size
//! - `builder-grid-size` - grid building routines over a range of grid sizes
//! - `fetch_icells` - `CMap2::iter_<CELL>` methods
//! - `grisubal` - grisubal kernel with a fixed size grid
//! - `grisubal-grid-size` - grisubal kernel over a range of grid granularity
//! - `triangulate-quads` - triangulate all cells of a mixed-mesh
//!
//! ### Iai-callgrind-based
//!
//! - `prof-dim2-basic` - `CMap2` basic operations benchmarks
//! - `prof-dim2-build` - `CMap2` constructor & building functions benchmarks
//! - `prof-dim2-sewing-unsewing` - `CMap2` (un)sewing & (un)linking methods benchmarks

// --- enable doc_auto_cfg feature if compiling in nightly
#![allow(unexpected_cfgs)]
#![cfg_attr(nightly, feature(doc_auto_cfg))]

#[doc(hidden)]
pub mod cli;
pub mod cut_edges;
pub mod grid_gen;
pub mod grisubal;
pub mod remesh;
pub mod shift;
#[doc(hidden)]
pub mod utils;
