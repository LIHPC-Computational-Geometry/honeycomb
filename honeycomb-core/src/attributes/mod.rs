//! attribute modeling code
//!
//! This module contains all code related to generic attribute modelling and handling.

mod collections;
mod manager;
mod traits;

pub use collections::AttrSparseVec;
pub use traits::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage};

pub(crate) use manager::AttrStorageManager;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AttributeError {
    #[error("cannot merge attributes: {0}")]
    FailedMerge(&'static str),
    #[error("cannot split attributes: {0}")]
    FailedSplit(&'static str),
    #[error("insufficient data to complete attribute operation: {0}")]
    InsufficientData(&'static str),
}

// ------ TESTS

#[allow(clippy::float_cmp)]
#[cfg(test)]
mod tests;
