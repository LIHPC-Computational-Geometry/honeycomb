//! # honeycomb-examples
//!
//! This crate contains all examples of the project.
//!
//! ## Available examples
//!
//! ### Utilities
//!
//! - `memory_usage` -- Save the memory usage of a given map as three *csv* files. These files
//!   can be used to generate charts using the `plot.py` script.
//!
//! ### Rendering
//!
//! - `render_default_no_aa` -- Render a hardcoded arrow without anti-aliasing.
//! - `render_default_smaa1x` -- Render a hardcoded arrow with anti-aliasing.
//! - `render_splitsquaremap` -- Render a map generated using functions defined in the `utils`
//!   module of the core crate
//! - `render_squaremap` -- Render a map generated using functions  defined in the `utils` module
//!   of the core crate.
//! - `render_squaremap_shift` -- Render a map computed by the `squaremap-shift` benchmark. Inner
//!   vertices are shifted by a random offset value.
//! - `render_squaremap_split_diff` -- Render a map computed by the `squaremap-splitquads`
//!   benchmark. All quads are split diagonally, which diagonal chosen at random.
//! - `render_squaremap_split_some` -- Render a map computed by the `squaremap-splitquads`
//!   benchmark. Only some quads are split diagonally, chosen at random.
