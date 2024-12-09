//! Main definitions
//!
//! This module contains the main structure definition ([`CMap3`]) as well as its constructor
//! implementation.

// ------ IMPORTS

use super::CMAP3_BETA;
use crate::{
    attributes::{AttrSparseVec, AttrStorageManager, UnknownAttributeStorage},
    cmap::components::{betas::BetaFunctions, unused::UnusedDarts},
    geometry::{CoordsFloat, Vertex3},
};

// ------ CONTENT

/// Main map object.
pub struct CMap3<T: CoordsFloat> {
    /// List of vertices making up the represented mesh
    pub(super) attributes: AttrStorageManager,
    /// List of vertices making up the represented mesh
    pub(super) vertices: AttrSparseVec<Vertex3<T>>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list
    pub(super) unused_darts: UnusedDarts,
    /// Array representation of the beta functions
    pub(super) betas: BetaFunctions<CMAP3_BETA>,
}

unsafe impl<T: CoordsFloat> Send for CMap3<T> {}
unsafe impl<T: CoordsFloat> Sync for CMap3<T> {}

#[doc(hidden)]
/// **Constructor convenience implementations**
impl<T: CoordsFloat> CMap3<T> {
    /// Creates a new 3D combinatorial map.
    #[allow(unused)]
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub(crate) fn new(n_darts: usize) -> Self {
        Self {
            attributes: AttrStorageManager::default(),
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts + 1),
            betas: BetaFunctions::new(n_darts + 1),
        }
    }

    /// Creates a new 3D combinatorial map with user-defined attributes
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub(crate) fn new_with_undefined_attributes(
        n_darts: usize,
        mut attr_storage_manager: AttrStorageManager,
    ) -> Self {
        // extend all storages to the expected length: n_darts + 1 (for the null dart)
        // the passed manager should be containing defined, empty storage i.e. attributes
        // are known, but no space has been used/allocated yet
        attr_storage_manager.extend_storages(n_darts + 1);
        Self {
            attributes: attr_storage_manager,
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts + 1),
            betas: BetaFunctions::new(n_darts + 1),
        }
    }
}
