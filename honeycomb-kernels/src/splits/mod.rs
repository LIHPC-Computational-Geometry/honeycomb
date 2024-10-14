//!

// ------ MODULE DECLARATIONS

// ------ PUBLIC RE-EXPORTS

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
