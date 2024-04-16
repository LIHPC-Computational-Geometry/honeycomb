//! Cell modeling code
//!
//! This module contains all code related to cell & orbit modeling.

// ------ MODULE DECLARATIONS

pub mod collections;
pub mod orbits;

// ------ CONTENT

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

pub const NULL_DART_ID: DartIdentifier = 0;
