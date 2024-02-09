//! Basic dart structure
//!
//! Useful definitions of this module being re-exported, the user should
//! most likely not interact directly with it.
//!
//! This module contains all code used to model darts as element of the
//! combinatorial map. This does not include any form of geometric
//! embedding as this is stored separately for the moment.

// ------ IMPORTS

// ------ CONTENT

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Dart object
///
/// Structure used to represent darts. This does not include any geometric
/// embedding.
///
/// # Example
///
/// ```
/// use honeycomb_core::Dart;
///
/// // Create a list containing:
/// // - the null dart
/// // - 10 darts with IDs ranging from one to 10 (included)
/// let mut darts = vec![Dart::NULL];
/// darts.extend((1..11).map(|i| Dart::from(i)));
///
/// (0..11).for_each(|i| assert_eq!(i, darts[i].id() as usize));
/// ```
///
pub struct Dart {
    id: DartIdentifier,
}

impl Dart {
    /// Null dart value for the structure. This is used as the returned
    /// value for the [Default] trait implementation.
    pub const NULL: Dart = Dart { id: 0 };

    /// Getter for the dart's identifier. This is preferred to making
    /// the `id` field public because there is currently no good reason
    /// to allow identifier overwriting.
    ///
    /// # Return / Panic
    ///
    /// Returns the identifier of the dart, of type [DartIdentifier]. A
    /// value of *0* implies that the dart is the null dart.
    ///
    /// # Example
    ///
    /// See structure example.
    ///
    pub fn id(&self) -> DartIdentifier {
        self.id
    }
}

impl From<DartIdentifier> for Dart {
    fn from(value: DartIdentifier) -> Self {
        Self { id: value }
    }
}

impl Default for Dart {
    fn default() -> Self {
        Self::NULL
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;
}
