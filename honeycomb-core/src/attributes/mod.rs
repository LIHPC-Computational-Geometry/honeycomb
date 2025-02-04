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
    #[error("cannot merge attribute {0}: {1}")]
    FailedMerge(&'static str, &'static str),
    #[error("cannot split attribute {0}: {1}")]
    FailedSplit(&'static str, &'static str),
    #[error("insufficient data to complete {0} operation on {1}")]
    InsufficientData(&'static str, &'static str),
}

// ------ TESTS

#[allow(clippy::float_cmp)]
#[cfg(test)]
mod tests;
