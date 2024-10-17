//! Crate-level common definitions

mod builder;
mod darts;

pub use builder::{BuilderError, CMapBuilder};
pub use darts::{DartIdentifier, NULL_DART_ID};

#[cfg(feature = "utils")]
pub use builder::GridDescriptor;
