//! Cell splitting functions
//!
//! This module contains implementations of cell splitting methods. We currently define
//! two edge-splitting methods, depending on the number of splits done. Both functions
//! have "no-alloc" variants: these take additional darts as argument in order not to
//! allocate darts during the process.

mod vertices;

pub use vertices::{VertexInsertionError, insert_vertex_in_edge, insert_vertices_in_edge};

#[cfg(test)]
mod tests;
