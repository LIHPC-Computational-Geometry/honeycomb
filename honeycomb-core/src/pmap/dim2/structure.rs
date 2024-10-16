use crate::attributes::{AttrSparseVec, UnknownAttributeStorage};
use crate::geometry::{CoordsFloat, Vertex2};
use crate::pmap::common::PDartIdentifier;
use crate::pmap::dim2::PMAP2_BETA;
use std::sync::atomic::{AtomicBool, AtomicU32};

// #[cfg_attr(feature = "utils", derive(Clone))]
pub struct PMap2<T: CoordsFloat> {
    // /// List of vertices making up the represented mesh
    // pub(super) attributes: AttrStorageManager,
    /// List of vertices making up the represented mesh
    pub(super) vertices: AttrSparseVec<Vertex2<T>>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list
    pub(super) unused_darts: Vec<AtomicBool>,
    /// Array representation of the beta functions
    pub(super) betas: Vec<[PDartIdentifier; PMAP2_BETA]>,
    /// Current number of darts
    pub(super) n_darts: usize,
}

#[doc(hidden)]
/// **Constructor convenience implementations**
impl<T: CoordsFloat> PMap2<T> {
    /// Creates a new 2D combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts composing the new combinatorial map.
    ///
    /// # Return
    ///
    /// Returns a combinatorial map containing `n_darts + 1` darts, the amount of darts wanted plus
    /// the null dart (at index `NULL_DART_ID` i.e. `0`).
    ///
    /// # Example
    ///
    /// See [`PMap2`] example.
    #[allow(unused)]
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub(crate) fn new(n_darts: usize) -> Self {
        Self {
            // attributes: AttrStorageManager::default(),
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: (0..=n_darts) // n_darts + 1
                .map(|_| AtomicBool::new(false))
                .collect(),
            betas: (0..=n_darts) // n_darts + 1
                .map(|_| {
                    [
                        AtomicU32::default(),
                        AtomicU32::default(),
                        AtomicU32::default(),
                    ]
                })
                .collect(),
            n_darts: n_darts + 1,
        }
    }
}
