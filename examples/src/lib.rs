//! # honeycomb-examples
//!
//! This crate contains all examples of the project.
//!
//! ## Available examples
//!
//! ### Input / Output
//!
//! - `io_read` -- Initialize and render a map from the VTK file passed to the command line.
//! - `io_write` -- Serialize the map that is built by the `squaremap_split_some` becnhmark.
//!   The file can be visualized using ParaView and compared to the render.
//!
//! ### Utilities
//!
//! - `memory_usage` -- Save the memory usage of a given map as three *csv* files. These files
//!   can be used to generate charts using the `plot.py` script.
//!
//! ### Rendering
//!
//! - `render` -- Render a map representing a simple orthogonal grid. Note that you may *need* to
//!   run this example in release mode if the input mesh is large.
