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

use std::sync::atomic::AtomicBool;

use crate::{FaceIdentifier, VertexIdentifier, VolumeIdentifier};

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

pub const NULL_DART_ID: DartIdentifier = 0;

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

#[derive(Clone, Copy, Debug, Default)]
/// Dart-cell associative structure
///
/// Structure used to store the associated vertex, face and volume
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
/// ```
/// use honeycomb_core::{cmap::dart::CellIdentifiers, Dart};
///
/// let darts = vec![Dart::NULL, Dart::from(1)];
/// let cell_ids = vec![CellIdentifiers::default(); 2];
///
/// println!("dart {} associated cells: {:#?}", darts[1].id(), cell_ids[darts[1].id() as usize]);
/// ```
///
pub struct CellIdentifiers {
    /// Vertex unique identifier.
    pub vertex_id: VertexIdentifier,
    /// Face unique identifier.
    pub face_id: FaceIdentifier,
    /// Volume unique identifier.
    pub volume_id: VolumeIdentifier,
}

/// Dart-associated data
///
/// This should not be used directly by the user.
///
/// Structure used to store dart-related data. The association of data with
/// a given dart is done implictly through indexing.
///
/// # Generics
///
/// - `const N_MARKS: usize` -- Number of marks used for search algorithms.
///   This corresponds to the number of search that can be done concurrently.
///
pub struct DartData<const N_MARKS: usize> {
    /// Array of boolean used for algorithmic search.
    pub marks: [Vec<AtomicBool>; N_MARKS],
    /// List of associated cell identifiers.
    pub associated_cells: Vec<CellIdentifiers>,
}

impl<const N_MARKS: usize> DartData<N_MARKS> {
    /// Create a DartData object.
    ///
    /// This should not be used directly by the user.
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
                    volume_id: 0
                };
                n_darts + 1
            ],
        }
    }

    /// Free all darts of a given mark.
    ///
    /// This should not be used directly by the user.
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
    /// - `dart_id: usize` -- Identifier of the dart that must be checked.
    ///
    /// # Return / Panic
    ///
    /// Returns a boolean to indicate whether the dart was marked or not. In
    /// both case, the dart is marked after the operation.
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
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;
}
