//! # honeycomb-benches
//!
//! This crate contains all benchmarks of the project. It also contains simple binaries used to
//! profile and further optimize the implementation.
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
//!
//! ## Available binaries
//!
//! - `builder` - Build a 2-map grid using dimensions passed as argument
//! - `grisubal` - Run the `grisubal` algorithm
//! - `shift` - Run a simple vertex relaxation algorithm in parallel (naively)
//! - `shift-nc` - Run a simple vertex relaxation algorithm in parallel (using independent set of
//!   vertices)

pub mod cli;
pub mod cut_edges;
pub mod grid_gen;
pub mod grisubal;
pub mod remesh;
pub mod shift;
pub mod utils;
