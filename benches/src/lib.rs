//! # honeycomb-benches
//!
//! This crate contains all benchmarks of the project. As a rule of thumb, the iai-callgrind
//! benchmarks cover individual methods of the structure while criterion benchmarks cover higher
//! level computations.
//!
//!
//! ## Available benchmarks
//!
//! ### Criterion-based
//!
//! - `builder` - grid building routines at fixed size
//! - `builder-grid-size` - grid building routines over a range of grid sizes
//! - `fetch_icells` - `CMap2::fetch_<CELL>` methods
//! - `grisubal` - grisubal kernel with a fixed size grid
//! - `grisubal-grid-size` - grisubal kernel over a range of grid granularity
//! - `triangulate-quads` - triangulate all cells of a mixed-mesh
//!
//! ### Iai-callgrind-based
//!
//! - `prof-dim2-basic` - `CMap2` basic operations benchmarks
//! - `prof-dim2-build` - `CMap2` constructor & building functions benchmarks
//! - `prof-dim2-sewing-unsewing` - `CMap2` (un)sewing & (un)linking methods benchmarks

cfg_if::cfg_if! {
    if #[cfg(feature = "_single_precision")] {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f32;
    } else {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f64;
    }
}
