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

use crate::{DartIdentifier, FaceIdentifier, VertexIdentifier};

// ------ CONTENT

#[derive(Clone, Copy, Debug, Default)]
/// Dart-cell associative structure
///
/// Structure used to store the associated vertex and face
/// identifiers to a dart. The structure technically contains only
/// cell identifiers, the association with a dart ID is done implicitly
/// through storage indexing.
///
/// Each field is kept public as editing operations can happen during
/// execution (e.g. a sewing operation will "fuse" some geometric
/// objects).
///
/// # Example
///
/// No example is provided as the structure should not be used directly.
/// The documentation is generated mostly for developing purposes.
///
pub struct CellIdentifiers {
    /// Vertex unique identifier.
    pub vertex_id: VertexIdentifier,
    /// Face unique identifier.
    pub face_id: FaceIdentifier,
}

/// Dart-associated data
///
/// **This should not be used directly by the user.**
///
/// Structure used to store dart-related data. The association of data with
/// a given dart is done implicitly through indexing.
///
/// # Example
///
/// No example is provided as the structure should not be used directly.
/// The documentation is generated mostly for developing purposes.
///
#[cfg_attr(feature = "benchmarking_utils", derive(Clone))]
pub struct DartData {
    /// List of associated cell identifiers.
    pub associated_cells: Vec<CellIdentifiers>,
}

impl DartData {
    /// Create a [DartData] object.
    ///
    /// **This should not be used directly by the user.**
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts in the new structure.
    ///
    /// # Return / Panic
    ///
    /// Returns a [DartData] structure with default values.
    ///
    pub fn new(n_darts: usize) -> Self {
        Self {
            associated_cells: vec![
                CellIdentifiers {
                    vertex_id: 0,
                    face_id: 0,
                    // volume_id: 0
                };
                n_darts + 1
            ],
        }
    }

    /// Add a new entry to the structure.
    ///
    /// **This should not be used directly by the user.**
    ///
    pub fn add_entry(&mut self) {
        self.associated_cells.push(CellIdentifiers::default());
    }

    /// Add multiple new entries to the structure.
    ///
    /// **This should not be used directly by the user.**
    ///
    pub fn add_entries(&mut self, n_darts: usize) {
        self.associated_cells
            .extend((0..n_darts).map(|_| CellIdentifiers::default()));
    }

    /// Reset a given entry of the structure.
    ///
    /// **This should not be used directly by the user.**
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of the dart which entry should be reset.
    ///
    pub fn reset_entry(&mut self, dart_id: DartIdentifier) {
        self.associated_cells[dart_id as usize] = CellIdentifiers::default();
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;
}
