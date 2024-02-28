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

use std::sync::atomic::AtomicBool;

use crate::{FaceIdentifier, VertexIdentifier};

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

pub const NULL_DART_ID: DartIdentifier = 0;

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
/// # Generics
///
/// - `const N_MARKS: usize` -- Number of marks used for search algorithms.
///   This corresponds to the number of search that can be done concurrently.
///
/// # Example
///
/// No example is provided as the structure should not be used directly.
/// The documentation is generated mostly for developing purposes.
///
pub struct DartData<const N_MARKS: usize> {
    /// Array of boolean used for algorithmic search.
    ///
    /// Atomics allow for non-mutable interfaces, i.e. parallel friendly
    /// methods. Storage is done line-wise as it would be very rare to
    /// access multiple marks of the same dart successively.
    pub marks: [Vec<AtomicBool>; N_MARKS],
    /// List of associated cell identifiers.
    pub associated_cells: Vec<CellIdentifiers>,
}

#[cfg(feature = "bench")]
impl<const N_MARKS: usize> Clone for DartData<N_MARKS> {
    fn clone(&self) -> Self {
        Self {
            marks: self
                .marks
                .iter()
                .map(|elem| {
                    elem.iter()
                        .map(|atombool| {
                            AtomicBool::new(atombool.load(std::sync::atomic::Ordering::Relaxed))
                        })
                        .collect::<Vec<AtomicBool>>()
                })
                .collect::<Vec<Vec<AtomicBool>>>()
                .try_into()
                .unwrap(),
            associated_cells: self.associated_cells.clone(),
        }
    }
}

impl<const N_MARKS: usize> DartData<N_MARKS> {
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
        let marks: [Vec<AtomicBool>; N_MARKS] = (0..N_MARKS)
            .map(|_| {
                (0..n_darts + 1)
                    .map(|_| AtomicBool::new(false))
                    .collect::<Vec<AtomicBool>>()
            })
            .collect::<Vec<Vec<AtomicBool>>>()
            .try_into()
            .unwrap();
        Self {
            marks,
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

    /// Free all darts of a given mark.
    ///
    /// **This should not be used directly by the user.**
    ///
    /// # Arguments
    ///
    /// - `mark_id: usize` -- Identifier of the mark that must be reset.
    ///
    pub fn free_mark(&self, mark_id: usize) {
        self.marks[mark_id]
            .iter()
            .filter(|e| e.load(std::sync::atomic::Ordering::Relaxed)) // useful?
            .for_each(|mark| mark.store(false, std::sync::atomic::Ordering::Relaxed));
    }

    /// Check and update if a given dart was marked.
    ///
    ///  # Arguments
    ///
    /// - `mark_id: usize` -- Identifier of the mark that must be checked.
    /// - `dart_id: DartIdentifier` -- Identifier of the dart that must be checked.
    ///
    /// # Return / Panic
    ///
    /// Returns a boolean to indicate whether the dart was marked or not. In
    /// both case, the dart is marked after the operation.
    ///
    /// The method will panic if the provided mark ID is invalid.
    ///
    pub fn was_marked(&self, mark_id: usize, dart_id: DartIdentifier) -> bool {
        assert!(mark_id < N_MARKS);
        match self.marks[mark_id][dart_id as usize].compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::Release,
            std::sync::atomic::Ordering::Relaxed,
        ) {
            Ok(_) => {
                // comparison was successful => was false
                false
            }
            Err(_) => {
                // comparison failed => was true
                true
            }
        }
    }

    /// Add a new entry to the structure.
    ///
    /// **This should not be used directly by the user.**
    ///
    pub fn add_entry(&mut self) {
        self.marks
            .iter_mut()
            .for_each(|mark| mark.push(AtomicBool::new(false)));
        self.associated_cells.push(CellIdentifiers::default());
    }

    /// Add multiple new entries to the structure.
    ///
    /// **This should not be used directly by the user.**
    ///
    pub fn add_entries(&mut self, n_darts: usize) {
        self.marks
            .iter_mut()
            .for_each(|mark| mark.extend((0..n_darts).map(|_| AtomicBool::new(false))));
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
        self.marks.iter().for_each(|mark| {
            mark[dart_id as usize].store(false, std::sync::atomic::Ordering::Relaxed)
        });
        self.associated_cells[dart_id as usize] = CellIdentifiers::default();
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;
}
