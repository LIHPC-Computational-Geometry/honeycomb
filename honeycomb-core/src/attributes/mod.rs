//! attribute modeling code
//!
//! This module contains all code related to generic attribute modelling and handling.

mod collections;
mod manager;
mod traits;

pub use collections::AttrSparseVec;
pub use traits::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage};

pub(crate) use manager::AttrStorageManager;

// ------ TESTS

#[allow(clippy::float_cmp)]
#[cfg(test)]
mod tests;
