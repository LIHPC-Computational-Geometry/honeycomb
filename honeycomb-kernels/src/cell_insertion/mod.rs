//! Cell insertion functions
//!
//! This module contains implementations of cell insertion methods. Due to definition issues, we
//! only implement insertion of `N-1`-cell into `N`-cell, for example vertex (0-cell) in edge
//! (1-cell).

mod vertices;

pub use vertices::{VertexInsertionError, insert_vertex_in_edge, insert_vertices_in_edge};

#[cfg(test)]
mod tests;
