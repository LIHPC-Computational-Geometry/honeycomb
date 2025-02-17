//! Cell splitting functions
//!
//! This module contains implementations of cell splitting methods. We currently define
//! two edge-splitting methods, depending on the number of splits done. Both functions
//! have "no-alloc" variants: these take additional darts as argument in order not to
//! allocate darts during the process.

mod edge_multiple;
mod edge_single;

pub use edge_multiple::{splitn_edge, splitn_edge_transac};
pub use edge_single::{split_edge, split_edge_transac};

use honeycomb_core::{
    attributes::AttributeError,
    cmap::{LinkError, SewError},
};

/// Error-modeling enum for edge-splitting routines.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SplitEdgeError {
    /// A core operation failed.
    #[error("core operation failed: {0}")]
    FailedCoreOp(#[from] SewError),
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
    #[error("wrong # of darts - expected `{0}`, got {1}")]
    WrongAmountDarts(usize, usize),
}

impl From<LinkError> for SplitEdgeError {
    fn from(value: LinkError) -> Self {
        Self::FailedCoreOp(value.into())
    }
}

impl From<AttributeError> for SplitEdgeError {
    fn from(value: AttributeError) -> Self {
        Self::FailedCoreOp(value.into())
    }
}

#[cfg(test)]
mod tests;
