//!

// ------ MODULE DECLARATIONS

mod edge_multiple;
mod edge_single;

// ------ PUBLIC RE-EXPORTS

pub use edge_multiple::{splitn_edge, splitn_edge_no_alloc};
pub use edge_single::{split_edge, split_edge_noalloc};

// ------ CONTENT

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SplitEdgeError {
    #[error("vertex placement for split is not in ]0;1[")]
    VertexBound,
    #[error("edge isn't defined correctly")]
    UndefinedEdge,
    #[error("passed darts should be free & non-null - {0}")]
    InvalidDarts(&'static str),
    #[error("not enough darts passed to the split function - missing `{0}`")]
    NotEnoughDarts(usize),
    #[error("too many darts passed to the split function - `{0}` too many")]
    TooManyDarts(usize),
}

// ------ TESTS

#[cfg(test)]
mod tests;
