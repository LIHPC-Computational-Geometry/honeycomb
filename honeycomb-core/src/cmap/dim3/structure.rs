//! Main definitions
//!
//! This module contains the main structure definition ([`CMap3`]) as well as its constructor
//! implementation.

use crate::{
    attributes::{AttrSparseVec, AttrStorageManager, UnknownAttributeStorage},
    cmap::{
        DartAllocError, DartIdType,
        components::{
            betas::BetaFunctions,
            darts::{CompactDartBlock, SparseDartBlock},
            unused::UnusedDarts,
        },
    },
    geometry::{CoordsFloat, Vertex3},
    stm::{StmClosureResult, Transaction, atomically},
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
    pub(crate) fn new(n_darts: usize) -> Self {
        Self {
            attributes: AttrStorageManager::default(),
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts + 1),
            betas: BetaFunctions::new(n_darts + 1),
        }
    }

    /// Creates a new 3D combinatorial map with user-defined attributes
    ///
    /// We expect the passed storages to be defined but empty, i.e. attributes are known,
    /// but no space has been used/ allocated yet.
    #[must_use = "unused return value"]
    pub(crate) fn new_with_undefined_attributes(
        n_darts: usize,
        mut attr_storage_manager: AttrStorageManager,
    ) -> Self {
        // extend all storages to the expected length: n_darts + 1 (for the null dart)
        attr_storage_manager.extend_storages(n_darts + 1);
        Self {
            attributes: attr_storage_manager,
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts + 1),
            betas: BetaFunctions::new(n_darts + 1),
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

    /// Return the current number of unused darts.
    #[must_use = "unused return value"]
    pub fn n_unused_darts(&self) -> usize {
        self.unused_darts.iter().filter(|v| v.read_atomic()).count()
    }

    /// Return whether a given dart is unused or not.
    #[must_use = "unused return value"]
    pub fn is_unused(&self, d: DartIdType) -> bool {
        self.unused_darts[d].read_atomic()
    }

    /// Return whether a given dart is unused or not.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    #[must_use = "unused return value"]
    pub fn is_unused_transac(
        &self,
        trans: &mut Transaction,
        d: DartIdType,
    ) -> StmClosureResult<bool> {
        self.unused_darts[d].read(trans)
    }

    /// Set a given dart as used.
    pub fn set_used(&self, d: DartIdType) {
        atomically(|t| self.set_used_transac(t, d));
    }

    /// Set a given dart as used.
    pub fn set_used_transac(&self, t: &mut Transaction, d: DartIdType) -> StmClosureResult<()> {
        self.unused_darts[d].write(t, false)
    }

    /// Set a given dart as unused.
    pub fn set_unused(&self, d: DartIdType) {
        atomically(|t| self.set_unused_transac(t, d));
    }

    /// Set a given dart as unused.
    pub fn set_unused_transac(&self, t: &mut Transaction, d: DartIdType) -> StmClosureResult<()> {
        self.unused_darts[d].write(t, true)
    }

    // --- edit

    /// Add new unused darts to the map.
    ///
    /// This method is meant to be used with `Self::reserve_compact_dart_block` and
    /// `Self::reserve_sparse_dart_block`. It currently allows allocations of any sizes, and
    /// block alignment is not enforced.
    ///
    /// This should eventually be replaced / completed with an `allocate_dart_blocks` method,
    /// which would handle both allocation and generation of block objects (the latter is currently
    /// done directly in the `reserve` methods).
    pub fn allocate_unused_darts(&mut self, n_darts: usize) {
        // self.n_darts += n_darts;
        self.betas.extend(n_darts);
        self.unused_darts.extend_from_val(n_darts, true);
        self.vertices.extend(n_darts);
        self.attributes.extend_storages(n_darts);
    }

    /// Attempt to reserve a contiguous block of `SIZE` free darts.
    ///
    /// # Return
    ///
    /// This method may return:
    /// - `Some(_)` if there were enough unused darts to build a contiguous block of `SIZE` darts,
    /// - `None` otherwise.
    ///
    /// The block object can be used to fetch the reserved darts, which were transactionally marked
    /// as used during the object's construction.
    #[must_use = "unused return value"]
    pub fn reserve_compact_dart_block<const SIZE: usize>(
        &self,
    ) -> Result<CompactDartBlock<SIZE>, DartAllocError> {
        CompactDartBlock::try_from(self)
    }

    /// Attempt to reserve `SIZE` free darts.
    ///
    /// # Return
    ///
    /// This method may return:
    /// - `Some(_)` if there were enough unused darts to build a block of `SIZE` darts,
    /// - `None` otherwise.
    ///
    /// The block object can be used to fetch the reserved darts, which were transactionally marked
    /// as used during the object's construction.
    #[must_use = "unused return value"]
    pub fn reserve_sparse_dart_block<const SIZE: usize>(
        &self,
    ) -> Result<SparseDartBlock<SIZE>, DartAllocError> {
        SparseDartBlock::try_from(self)
    }

    /// Add a new free dart to the map.
    ///
    /// # Return
    ///
    /// Returns the ID of the new dart.
    pub fn add_free_dart(&mut self) -> DartIdType {
        let new_id = self.n_darts() as DartIdType;
        self.betas.extend(1);
        self.unused_darts.extend(1);
        self.vertices.extend(1);
        self.attributes.extend_storages(1);
        new_id
    }

    /// Add `n_darts` new free darts to the map.
    ///
    /// # Return
    ///
    /// Returns the ID of the first new dart. Other IDs are in the range `ID..ID+n_darts`.
    pub fn add_free_darts(&mut self, n_darts: usize) -> DartIdType {
        let new_id = self.n_darts() as DartIdType;
        self.betas.extend(n_darts);
        self.unused_darts.extend(n_darts);
        self.vertices.extend(n_darts);
        self.attributes.extend_storages(n_darts);
        new_id
    }

    /// Insert a new free dart into the map.
    ///
    /// This method attempts to reuse an unused dart slot if available; otherwise, it adds a new one.
    ///
    /// # Return
    ///
    /// Returns the ID of the inserted dart.
    pub fn insert_free_dart(&mut self) -> DartIdType {
        if let Some((new_id, _)) = self
            .unused_darts
            .iter()
            .enumerate()
            .find(|(_, u)| u.read_atomic())
        {
            atomically(|trans| self.unused_darts[new_id as DartIdType].write(trans, false));
            new_id as DartIdType
        } else {
            self.add_free_dart()
        }
    }

    /// Remove a free dart from the map.
    ///
    /// The removed dart identifier is added to the list of free darts. This way of proceeding is
    /// necessary as the structure relies on dart indexing for encoding data, making reordering of
    /// any sort extremely costly.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdType` -- Identifier of the dart to remove.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - the dart is not free for all *i*,
    /// - the dart is already marked as unused.
    pub fn remove_free_dart(&mut self, dart_id: DartIdType) {
        atomically(|trans| {
            assert!(self.is_free(dart_id)); // all beta images are 0
            assert!(!self.unused_darts[dart_id as DartIdType].replace(trans, true)?);
            Ok(())
        });
    }
}
