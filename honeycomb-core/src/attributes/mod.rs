//! attribute modeling code
//!
//! This module contains all code related to generic attribute modelling and handling.

// ------ MODULE DECLARATIONS

mod collections;
mod manager;
mod traits;

pub use collections::AttrSparseVec;
pub use manager::AttrStorageManager;
pub use traits::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage};

// ------ TESTS

#[cfg(test)]
mod tests;
