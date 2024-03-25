//! Basic dart utilities
//!
//! **Useful definitions of this module being re-exported, the user should
//! most likely not interact directly with it.**
//!
//! This module contains all code used to model darts as element of the
//! combinatorial map. This includes geometric embedding as associated
//! identifiers, though spatial representation is left to another part
//! of the crate.

// ------ IMPORTS

// ------ CONTENT

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

pub const NULL_DART_ID: DartIdentifier = 0;

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;
}
