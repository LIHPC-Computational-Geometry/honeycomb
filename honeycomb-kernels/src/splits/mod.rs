//!

// ------ MODULE DECLARATIONS

mod edge_multiple;
mod edge_single;

// ------ PUBLIC RE-EXPORTS

pub use edge_multiple::{splitn_edge, splitn_edge_no_alloc};
pub use edge_single::{split_edge, split_edge_noalloc};

// ------ CONTENT

/// Error-modeling enum for edge-splitting routines.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SplitEdgeError {
    /// Relative position of the new vertex isn't located on the edge.
    #[error("vertex placement for split is not in ]0;1[")]
    VertexBound,
    /// One or both vertices of the edge are undefined.
    #[error("edge isn't defined correctly")]
    UndefinedEdge,
    /// Darts passed to the function do not match requirements.
    #[error("passed darts should be free & non-null - {0}")]
    InvalidDarts(&'static str),
    /// The number of darts passed to create the new segments is too low. The `usize` value
    /// is the number of missing darts.
    #[error("not enough darts passed to the split function - missing `{0}`")]
    NotEnoughDarts(usize),
    /// The number of darts passed to create the new segments is too high. The `usize` value
    /// is the number of excess darts.
    #[error("too many darts passed to the split function - `{0}` too many")]
    TooManyDarts(usize),
}

// ------ TESTS

#[cfg(test)]
mod tests;
