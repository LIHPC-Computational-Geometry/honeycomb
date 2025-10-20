//! Main definitions
//!
//! This module contains the main structure definition ([`CMap2`]) as well as its constructor
//! implementation.

use crate::attributes::{AttrSparseVec, AttrStorageManager, UnknownAttributeStorage};
use crate::cmap::{
    DartIdType, DartReleaseError, DartReservationError,
    components::{betas::BetaFunctions, unused::UnusedDarts},
};
use crate::geometry::{CoordsFloat, Vertex2};
use crate::stm::{
    StmClosureResult, Transaction, TransactionClosureResult, abort, atomically_with_err,
};

use super::CMAP2_BETA;

/// # 2D combinatorial map implementation
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
/// - We chose a boundary-less representation of meshes (i.e. darts on the boundary are 2-free).
/// - The null dart will always be encoded as `0`.
///
/// ## Generics
///
/// - `T: CoordsFloat` -- Generic FP type for coordinates representation
///
/// ## Example
///
/// The following code corresponds to this flow of operations:
///
/// ![`CMAP2_EXAMPLE`](https://lihpc-computational-geometry.github.io/honeycomb/user-guide/images/bg_hcmap_example.svg)
///
/// Note that:
/// - we create the map using its builder structure: [`CMapBuilder`][crate::cmap::CMapBuilder]
/// - we insert a few assertions to demonstrate the progressive changes applied to the structure
/// - even though the faces are represented in the figure, they are not stored in the structure
/// - we use a lot of methods with the `force_` prefix; these are convenience methods when
///   synchronization isn't needed
///
/// ```
/// # fn main() {
/// use honeycomb_core::{
///     cmap::{CMap2, CMapBuilder, OrbitPolicy},
///     geometry::Vertex2
/// };
///
/// // build a triangle (A)
/// let mut map: CMap2<f64> = CMapBuilder::<2>::from_n_darts(3).build().unwrap(); // three darts
/// map.force_link::<1>(1, 2); // beta1(1) = 2 & beta0(2) = 1
/// map.force_link::<1>(2, 3); // beta1(2) = 3 & beta0(3) = 2
/// map.force_link::<1>(3, 1); // beta1(3) = 1 & beta0(1) = 3
/// map.force_write_vertex(1, (0.0, 0.0));
/// map.force_write_vertex(2, (1.0, 0.0));
/// map.force_write_vertex(3, (0.0, 1.0));
///
/// // we can go through the face using an orbit
/// {
///     let mut face = map.orbit(OrbitPolicy::Face, 1);
///     assert_eq!(face.next(), Some(1));
///     assert_eq!(face.next(), Some(2));
///     assert_eq!(face.next(), Some(3));
///     assert_eq!(face.next(), None);
/// }
///
/// // build a second triangle (B)
/// let first_added_dart_id = map.allocate_used_darts(3);
/// assert_eq!(first_added_dart_id, 4);
/// map.force_link::<1>(4, 5);
/// map.force_link::<1>(5, 6);
/// map.force_link::<1>(6, 4);
/// map.force_write_vertex(4, (0.0, 2.0));
/// map.force_write_vertex(5, (2.0, 0.0));
/// map.force_write_vertex(6, (1.0, 1.0));
///
/// // there should be two faces now
/// let faces: Vec<_> = map.iter_faces().collect();
/// assert_eq!(&faces, &[1, 4]);
///
/// // sew both triangles (C)
/// map.force_sew::<2>(2, 4);
///
/// // there are 5 edges now, making up a square & its diagonal
/// let edges: Vec<_> = map.iter_edges().collect();
/// assert_eq!(&edges, &[1, 2, 3, 5, 6]);
///
/// // adjust bottom-right & top-left vertex position (D)
/// assert_eq!(
///     map.force_write_vertex(2, Vertex2::from((1.0, 0.0))),
///     Some(Vertex2(1.5, 0.0)) // `write` act as a `replace`
/// );
/// assert_eq!(
///     map.force_write_vertex(3, Vertex2::from((0.0, 1.0))),
///     Some(Vertex2(0.0, 1.5)) // these values were the average of sewn vertices
/// );
///
/// // separate the diagonal from the rest (E)
/// map.force_unsew::<1>(1);
/// map.force_unsew::<1>(2);
/// map.force_unsew::<1>(6);
/// map.force_unsew::<1>(4);
/// // break up & remove the diagonal
/// map.force_unsew::<2>(2); // this makes dart 2 and 4 free
/// map.release_dart(2);
/// map.release_dart(4);
/// // sew the square back up
/// map.force_sew::<1>(1, 5);
/// map.force_sew::<1>(6, 3);
///
/// // there's only the square face left
/// let faces: Vec<_> = map.iter_faces().collect();
/// assert_eq!(&faces, &[1]);
/// // we can check the vertices
/// let vertices = map.iter_vertices();
/// let mut value_iterator = vertices.map(|vertex_id| map.force_read_vertex(vertex_id).unwrap());
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((0.0, 0.0)))); // vertex ID 1
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((0.0, 1.0)))); // vertex ID 3
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((1.0, 0.0)))); // vertex ID 5
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((1.0, 1.0)))); // vertex ID 6
/// # }
/// ```
pub struct CMap2<T: CoordsFloat> {
    /// List of vertices making up the represented mesh
    pub(super) attributes: AttrStorageManager,
    /// List of vertices making up the represented mesh
    pub(super) vertices: AttrSparseVec<Vertex2<T>>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list
    pub(super) unused_darts: UnusedDarts,
    /// Array representation of the beta functions
    pub(super) betas: BetaFunctions<CMAP2_BETA>,
    /// Current number of darts
    pub(super) n_darts: usize,
}

unsafe impl<T: CoordsFloat> Send for CMap2<T> {}
unsafe impl<T: CoordsFloat> Sync for CMap2<T> {}

#[doc(hidden)]
/// **Constructor convenience implementations**
impl<T: CoordsFloat> CMap2<T> {
    /// Creates a new 2D combinatorial map.
    #[allow(unused)]
    #[must_use = "unused return value"]
    pub(crate) fn new(n_darts: usize) -> Self {
        Self {
            attributes: AttrStorageManager::default(),
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts + 1),
            betas: BetaFunctions::new(n_darts + 1),
            n_darts: n_darts + 1,
        }
    }

    /// Creates a new 2D combinatorial map with user-defined attributes
    ///
    /// We expect the passed storages to be defined but empty, i.e. attributes are known,
    /// but no space has been used/  allocated yet.
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
            n_darts: n_darts + 1,
        }
    }
}

/// **Dart-related methods**
impl<T: CoordsFloat> CMap2<T> {
    // --- read

    /// Return the current number of darts.
    #[must_use = "unused return value"]
    pub fn n_darts(&self) -> usize {
        self.n_darts
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
    pub fn is_unused_tx(&self, t: &mut Transaction, d: DartIdType) -> StmClosureResult<bool> {
        self.unused_darts[d].read(t)
    }

    // --- allocation

    /// Add `n_darts` new free darts to the map.
    ///
    /// This is an internal helper function
    fn allocate_darts_core(&mut self, n_darts: usize, unused: bool) -> DartIdType {
        let new_id = self.n_darts as DartIdType;
        self.n_darts += n_darts;
        self.betas.extend(n_darts);
        self.unused_darts.extend_with(n_darts, unused);
        self.vertices.extend(n_darts);
        self.attributes.extend_storages(n_darts);
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
        Ok(self.unused_darts[dart_id].replace(t, true)?) // Ok(_?) necessary for err type coercion
    }
}
