//! Main definitions
//!
//! This module contains the main structure definition ([`CMap3`]) as well as its constructor
//! implementation.

use fast_stm::TVar;
#[cfg(feature = "par-internals")]
use rayon::prelude::*;

use crate::{
    attributes::{AttrSparseVec, AttrStorageManager, UnknownAttributeStorage},
    cmap::{
        DartIdType, DartReleaseError, DartReservationError, VertexIdType,
        components::{betas::BetaFunctions, unused::UnusedDarts},
    },
    geometry::{CoordsFloat, Vertex3},
    stm::{StmClosureResult, Transaction, TransactionClosureResult, abort, atomically_with_err},
};

use super::CMAP3_BETA;

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
    pub(super) vid_cache: Option<Vec<TVar<VertexIdType>>>,
}

unsafe impl<T: CoordsFloat> Send for CMap3<T> {}
unsafe impl<T: CoordsFloat> Sync for CMap3<T> {}
#[doc(hidden)]
/// # 3D combinatorial map implementation
///
/// Information regarding maps can be found in the [user guide][UG].
/// This documentation focuses on the implementation and its API.
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/definitions/cmaps
///
/// Notes on implementation:
/// - We encode *β<sub>0</sub>* as the inverse function of *β<sub>1</sub>*. This is extremely
///   useful (read *required*) to implement correct and efficient i-cell computation. Additionally,
///   while *β<sub>0</sub>* can be accessed using the [`beta`][Self::beta] method, we do not define
///   the 0-sew / 0-unsew operations.
/// - We chose a boundary-less representation of meshes (i.e. darts on the boundary are 3-free).
/// - The null dart will always be encoded as `0`.
///
/// ## Generics
///
/// - `T: CoordsFloat` -- Generic FP type for coordinates representation
///
/// ## Example
///
/// The following example corresponds to this flow of operations:
///
/// - Building a tetrahedron (A)
/// - Building another tetrahedron (B)
/// - Sewing both tetrahedrons along a face (C)
/// - Adjusting shared vertices (D)
/// - Separating and removing the shared face (E)
///
/// ```rust
/// # fn main() {
/// // TODO: complete with test example once the structure is integrated to the builder
/// # }
/// ```
///
/// Note that:
/// - We use the builder structure: [`CMapBuilder`][crate::prelude::CMapBuilder]
/// - We insert a few assertions to demonstrate the progressive changes applied to the structure
/// - Even though volumes are represented in the figure, they are not stored in the structure
/// - We use a lot of methods with the `force_` prefix; these are convenience methods when
///   synchronization isn't needed
impl<T: CoordsFloat> CMap3<T> {
    /// Creates a new 3D combinatorial map.
    #[allow(unused)]
    #[must_use = "unused return value"]
    pub(crate) fn new(n_darts: usize, enable_vid_cache: bool) -> Self {
        Self {
            attributes: AttrStorageManager::default(),
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts + 1),
            betas: BetaFunctions::new(n_darts + 1),
            vid_cache: if enable_vid_cache {
                Some(
                    (0..n_darts as VertexIdType + 1)
                        .map(|v| TVar::new(v))
                        .collect(),
                )
            } else {
                None
            },
        }
    }

    /// Creates a new 3D combinatorial map with user-defined attributes
    ///
    /// We expect the passed storages to be defined but empty, i.e. attributes are known,
    /// but no space has been used/ allocated yet.
    #[must_use = "unused return value"]
    pub(crate) fn new_with_undefined_attributes(
        n_darts: usize,
        enable_vid_cache: bool,
        mut attr_storage_manager: AttrStorageManager,
    ) -> Self {
        // extend all storages to the expected length: n_darts + 1 (for the null dart)
        attr_storage_manager.extend_storages(n_darts + 1);
        Self {
            attributes: attr_storage_manager,
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts + 1),
            betas: BetaFunctions::new(n_darts + 1),
            vid_cache: if enable_vid_cache {
                Some(
                    (0..n_darts as VertexIdType + 1)
                        .map(|v| TVar::new(v))
                        .collect(),
                )
            } else {
                None
            },
        }
    }
}

/// **Dart-related methods**
impl<T: CoordsFloat> CMap3<T> {
    // --- read

    /// Return the current number of darts.
    #[must_use = "unused return value"]
    pub fn n_darts(&self) -> usize {
        self.unused_darts.len()
    }

    #[cfg(not(feature = "par-internals"))]
    /// Return the current number of unused darts.
    #[must_use = "unused return value"]
    pub fn n_unused_darts(&self) -> usize {
        self.unused_darts.iter().filter(|v| v.read_atomic()).count()
    }

    #[cfg(feature = "par-internals")]
    /// Return the current number of unused darts.
    #[must_use = "unused return value"]
    pub fn n_unused_darts(&self) -> usize {
        self.unused_darts
            .par_iter()
            .filter(|v| v.read_atomic())
            .count()
    }

    /// Return whether a given dart is unused or not.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    #[must_use = "unused return value"]
    pub fn is_unused_tx(&self, t: &mut Transaction, d: DartIdType) -> StmClosureResult<bool> {
        self.unused_darts[d].read(t)
    }

    // --- edit

    /// Add `n_darts` new free darts to the map.
    fn allocate_darts_core(&mut self, n_darts: usize, unused: bool) -> DartIdType {
        let new_id = self.n_darts() as DartIdType;
        self.betas.extend(n_darts);
        self.unused_darts.extend_with(n_darts, unused);
        self.vertices.extend(n_darts);
        self.attributes.extend_storages(n_darts);
        if let Some(ref mut vids) = self.vid_cache {
            vids.extend((new_id..new_id + n_darts as VertexIdType).map(|v| TVar::new(v)));
        }
        new_id
    }

    /// Add `n_darts` new free darts to the map.
    ///
    /// Added darts are marked as used.
    ///
    /// # Return
    ///
    /// Return the ID of the first new dart. Other IDs are in the range `ID..ID+n_darts`.
    pub fn allocate_used_darts(&mut self, n_darts: usize) -> DartIdType {
        self.allocate_darts_core(n_darts, false)
    }

    /// Add `n_darts` new free darts to the map.
    ///
    /// Added dart are marked as unused.
    ///
    /// # Return
    ///
    /// Return the ID of the first new dart. Other IDs are in the range `ID..ID+n_darts`.
    pub fn allocate_unused_darts(&mut self, n_darts: usize) -> DartIdType {
        self.allocate_darts_core(n_darts, true)
    }

    // --- reservation / removal

    #[allow(clippy::missing_errors_doc)]
    /// Mark `n_darts` free darts as used and return them for usage.
    ///
    /// # Return / Errors
    ///
    /// This function returns a vector containing IDs of the darts marked as used. It will fail if
    /// there are not enough unused darts to return; darts will then be left as unused.
    pub fn reserve_darts(&self, n_darts: usize) -> Result<Vec<DartIdType>, DartReservationError> {
        atomically_with_err(|t| self.reserve_darts_tx(t, n_darts))
    }

    #[allow(clippy::missing_errors_doc)]
    /// Mark `n_darts` free darts as used and return them for usage.
    ///
    /// # Return / Errors
    ///
    /// This function returns a vector containing IDs of the darts marked as used. It will fail if
    /// there are not enough unused darts to return; darts will then be left as unused.
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn reserve_darts_tx(
        &self,
        t: &mut Transaction,
        n_darts: usize,
    ) -> TransactionClosureResult<Vec<DartIdType>, DartReservationError> {
        let mut res = Vec::with_capacity(n_darts);

        for d in 1..self.n_darts() as DartIdType {
            if self.is_unused_tx(t, d)? {
                self.claim_dart_tx(t, d)?;
                res.push(d);
                if res.len() == n_darts {
                    return Ok(res);
                }
            }
        }

        abort(DartReservationError(n_darts))
    }

    #[allow(clippy::missing_errors_doc)]
    /// Mark `n_darts` free darts as used and return them for usage.
    ///
    /// While `reserve_darts_tx` search for free darts from dart 1, this function takes as argument
    /// a dart ID which serve as the starting point of the search. This is useful in parallel
    /// contexts; multiple threads may use different offsets to reserve darts without competing
    /// repeatedly to claim the same elements.
    ///
    /// # Return / Errors
    ///
    /// This function returns a vector containing IDs of the darts marked as used. It will fail if
    /// there are not enough unused darts to return; darts will then be left as unused.
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn reserve_darts_from_tx(
        &self,
        t: &mut Transaction,
        n_darts: usize,
        from: DartIdType,
    ) -> TransactionClosureResult<Vec<DartIdType>, DartReservationError> {
        let mut res = Vec::with_capacity(n_darts);

        for d in (from..self.n_darts() as DartIdType).chain(1..from) {
            if self.is_unused_tx(t, d)? {
                self.claim_dart_tx(t, d)?;
                res.push(d);
                if res.len() == n_darts {
                    return Ok(res);
                }
            }
        }

        abort(DartReservationError(n_darts))
    }

    /// Set a given dart as used.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn claim_dart_tx(&self, t: &mut Transaction, dart_id: DartIdType) -> StmClosureResult<()> {
        self.unused_darts[dart_id].write(t, false)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Mark a free dart from the map as unused.
    ///
    /// # Return / Errors
    ///
    /// This method return a boolean indicating whether the art was already unused or not. It will
    /// fail if the dart is not free, i.e. if one of its beta images isn't null.
    pub fn release_dart(&self, dart_id: DartIdType) -> Result<bool, DartReleaseError> {
        atomically_with_err(|t| self.release_dart_tx(t, dart_id))
    }

    #[allow(clippy::missing_errors_doc)]
    /// Mark a free dart from the map as unused.
    ///
    /// # Return / Errors
    ///
    /// This method return a boolean indicating whether the art was already unused or not. It will
    /// fail if the dart is not free, i.e. if one of its beta images isn't null.
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn release_dart_tx(
        &self,
        t: &mut Transaction,
        dart_id: DartIdType,
    ) -> TransactionClosureResult<bool, DartReleaseError> {
        if !self.is_free_tx(t, dart_id)? {
            abort(DartReleaseError(dart_id))?;
        }
        self.attributes.clear_attribute_values(t, dart_id)?;
        self.vertices.clear_slot(t, dart_id)?;
        if let Some(ref vids) = self.vid_cache {
            vids[dart_id as usize].write(t, dart_id)?;
        }
        Ok(self.unused_darts[dart_id].replace(t, true)?) // Ok(_?) necessary for err type coercion
    }
}
