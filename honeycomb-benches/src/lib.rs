//! # honeycomb-benches
//!
//! This crate contains all benchmarks of the project. As a rule of thumb, the iai-callgrind
//! benchmarks cover individual methods of the structure while criterion benchmarks cover higher
//! level computations.
//!
//! ## Available benchmarks
//!
//! ### Criterion-based
//!
//! - `splitsquaremap-init` - measures construction speed of the CMap2 structure
//! - `splitsquaremap-shift` - measures coordinate shifting speed in the CMap2 structure
//! - `squaremap-init` - construction speed of the CMap2 structure
//! - `squaremap-shift` - measures coordinate shifting speed in the CMap2 structure
//! - `squaremap-splitquads` - measures operation speed for quad to triangle transformation
//!
//! ### Iai-callgrind-based
//!
//! - `prof-cmap2-editing` - `CMap2` editing methods benchmarks
//! - `prof-cmap2-reading` - `CMap2` reading methods benchmarks
//! - `prof-cmap2-sewing-unsewing` - `CMap2` (un)sewing methods benchmarks
