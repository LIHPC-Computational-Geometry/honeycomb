//! # honeycomb-examples
//!
//! This crate contains all examples of the project.
//!
//! ## Available examples
//!
//! ### Algorithm
//!
//! - `grisubal` -- Run the grisubal algorithm on a specified input.
//! - `parallel_shift` -- Run a simple parallel vertex relaxation routine that highlights usage of
//!   the STM model.
//!
//! ### Input / Output
//!
//! - `io_read` -- Initialize and render a map from the VTK file passed to the command line.
//! - `io_write` -- Serialize a map representing a grid with quads randomly split diagonally.
//!   The output file can be visualized using ParaView and compared to the render.
//!
//! ### Rendering
//!
//! - `render` -- Render a map representing a simple orthogonal grid. Note that you may *need* to
//!   run this example in release mode.
