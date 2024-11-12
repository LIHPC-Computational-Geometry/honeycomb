//! Main definitions
//!
//! This module contains the main structure definition ([`CMap2`]) as well as its constructor
//! implementation.

// ------ IMPORTS

use super::CMAP2_BETA;
use crate::cmap::components::betas::BetaFunctions;
use crate::cmap::components::unused::UnusedDarts;
use crate::prelude::Vertex2;
use crate::{
    attributes::{AttrSparseVec, AttrStorageManager, UnknownAttributeStorage},
    geometry::CoordsFloat,
};

// ------ CONTENT

/// Main map object.
///
/// Structure used to model 2D combinatorial map. The structure implements basic operations as
/// well as higher level abstractions that are useful to write meshing applications.
///
/// Definition of the structure and its logic can be found in the [user guide][UG].
/// This documentation focuses on the implementation and its API.
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/definitions/cmaps
///
/// <div class="warning">
///
/// **This structure only implements `Clone` if the `utils` feature is enabled.**
///
/// </div>
///
/// # Fields
///
/// Fields are kept private in order to better define interfaces. The structure
/// contains the following data:
///
/// - an array-based representation of the beta functions
/// - a storage structure for vertices making up the represented mesh
/// - a generic storage manager for user-defined attributes
///
/// Note that:
/// - we encode *β<sub>0</sub>* as the inverse function of *β<sub>1</sub>*. This is extremely
///   useful (read *required*) to implement correct and efficient i-cell computation. Additionally,
///   while *β<sub>0</sub>* can be accessed using the [`Self::beta`] method, we do not define
///   the 0-sew / 0-unsew operations.
/// - we chose a boundary-less representation of meshes (i.e. darts on the boundary are 2-free).
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type for coordinates representation.
///
/// # Example
///
/// The following example goes over multiple operations on the mesh in order
/// to demonstrate general usage of the structure and its methods.
///
/// ![`CMAP2_EXAMPLE`](https://lihpc-computational-geometry.github.io/honeycomb/images/bg_hcmap_example.svg)
///
/// Note that:
/// - we create the map using its builder structure: [`CMapBuilder`][crate::prelude::CMapBuilder].
/// - the map we operate on has no boundaries. In addition to the different
///   operations realized at each step, we insert a few assertions to demonstrate the
///   progressive changes applied to the structure.
/// - in a real application, you will likely want to initialize the map from a serialized mesh file.
///
/// ```
/// # fn main() {
/// use honeycomb_core::prelude::{CMap2, CMapBuilder, DartId, EdgeId, FaceId, Orbit2, OrbitPolicy, VertexId, Vertex2};
///
/// let mut map: CMap2<f64> = CMapBuilder::default().n_darts(3).build().unwrap();
/// map.one_link(DartId(1), DartId(2));
/// map.one_link(DartId(2), DartId(3));
/// map.one_link(DartId(3), DartId(1));
/// map.insert_vertex(VertexId(1), (0.0, 0.0));
/// map.insert_vertex(VertexId(2), (1.0, 0.0));
/// map.insert_vertex(VertexId(3), (0.0, 1.0));
///
/// // checks
/// let faces = map.fetch_faces();
/// assert_eq!(faces.identifiers.len(), 1);
/// assert_eq!(faces.identifiers[0], FaceId(1));
/// let mut face = Orbit2::new(&map, OrbitPolicy::Face, DartId(1));
/// assert_eq!(face.next(), Some(DartId(1)));
/// assert_eq!(face.next(), Some(DartId(2)));
/// assert_eq!(face.next(), Some(DartId(3)));
/// assert_eq!(face.next(), None);
///
/// // build a second triangle
/// map.add_free_darts(3);
/// map.one_link(DartId(4), DartId(5));
/// map.one_link(DartId(5), DartId(6));
/// map.one_link(DartId(6), DartId(4));
/// map.insert_vertex(VertexId(4), (0.0, 2.0));
/// map.insert_vertex(VertexId(5), (2.0, 0.0));
/// map.insert_vertex(VertexId(6), (1.0, 1.0));
///
/// // checks
/// let faces = map.fetch_faces();
/// assert_eq!(&faces.identifiers, &[FaceId(1), FaceId(4)]);
/// let mut face = Orbit2::new(&map, OrbitPolicy::Face, DartId(4));
/// assert_eq!(face.next(), Some(DartId(4)));
/// assert_eq!(face.next(), Some(DartId(5)));
/// assert_eq!(face.next(), Some(DartId(6)));
/// assert_eq!(face.next(), None);
///
/// // sew both triangles
/// map.two_sew(DartId(2), DartId(4));
///
/// // checks
/// assert_eq!(map.beta::<2>(DartId(2)), DartId(4));
/// assert_eq!(map.vertex_id(DartId(2)), VertexId(2));
/// assert_eq!(map.vertex_id(DartId(5)), VertexId(2));
/// assert_eq!(map.vertex(VertexId(2)).unwrap(), Vertex2::from((1.5, 0.0)));
/// assert_eq!(map.vertex_id(DartId(3)), VertexId(3));
/// assert_eq!(map.vertex_id(DartId(4)), VertexId(3));
/// assert_eq!(map.vertex(VertexId(3)).unwrap(), Vertex2::from((0.0, 1.5)));
/// let edges = map.fetch_edges();
/// assert_eq!(
///     &edges.identifiers,
///     &[EdgeId(1), EdgeId(2), EdgeId(3), EdgeId(5), EdgeId(6)]
/// );
///
/// // adjust bottom-right & top-left vertex position
/// assert_eq!(
///     map.replace_vertex(VertexId(2), Vertex2::from((1.0, 0.0))),
///     Some(Vertex2::from((1.5, 0.0)))
/// );
/// assert_eq!(map.vertex(VertexId(2)).unwrap(), Vertex2::from((1.0, 0.0)));
/// assert_eq!(
///     map.replace_vertex(VertexId(3), Vertex2::from((0.0, 1.0))),
///     Some(Vertex2::from((0.0, 1.5)))
/// );
/// assert_eq!(map.vertex(VertexId(3)).unwrap(), Vertex2::from((0.0, 1.0)));
///
/// // separate the diagonal from the rest
/// map.one_unsew(DartId(1));
/// map.one_unsew(DartId(2));
/// map.one_unsew(DartId(6));
/// map.one_unsew(DartId(4));
/// // break up & remove the diagonal
/// map.two_unsew(DartId(2)); // this makes dart 2 and 4 free
/// map.remove_free_dart(DartId(2));
/// map.remove_free_dart(DartId(4));
/// // sew the square back up
/// map.one_sew(DartId(1), DartId(5));
/// map.one_sew(DartId(6), DartId(3));
///
/// // i-cells
/// let faces = map.fetch_faces();
/// assert_eq!(&faces.identifiers, &[FaceId(1)]);
/// let edges = map.fetch_edges();
/// assert_eq!(
///     &edges.identifiers,
///     &[EdgeId(1), EdgeId(3), EdgeId(5), EdgeId(6)]
/// );
/// let vertices = map.fetch_vertices();
/// assert_eq!(
///     &vertices.identifiers,
///     &[VertexId(1), VertexId(3), VertexId(5), VertexId(6)]
/// );
/// assert_eq!(map.vertex(VertexId(1)).unwrap(), Vertex2::from((0.0, 0.0)));
/// assert_eq!(map.vertex(VertexId(5)).unwrap(), Vertex2::from((1.0, 0.0)));
/// assert_eq!(map.vertex(VertexId(6)).unwrap(), Vertex2::from((1.0, 1.0)));
/// assert_eq!(map.vertex(VertexId(3)).unwrap(), Vertex2::from((0.0, 1.0)));
/// // darts
/// assert_eq!(map.n_unused_darts(), 2); // there are unused darts since we removed the diagonal
/// assert_eq!(map.beta_runtime(1, DartId(1)), DartId(5));
/// assert_eq!(map.beta_runtime(1, DartId(5)), DartId(6));
/// assert_eq!(map.beta_runtime(1, DartId(6)), DartId(3));
/// assert_eq!(map.beta_runtime(1, DartId(3)), DartId(1));
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
    /// See [`CMap2`] example.
    #[allow(unused)]
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub(crate) fn new(n_darts: usize) -> Self {
        Self {
            attributes: AttrStorageManager::default(),
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: UnusedDarts::new(n_darts),
            betas: BetaFunctions::new(n_darts),
            n_darts: n_darts + 1,
        }
    }

    /// Creates a new 2D combinatorial map with user-defined attributes
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts composing the new combinatorial map.
    /// - `attr_storage_manager: AttrStorageManager` -- Manager structure holding
    ///   the user-defined attributes. The containers held by the manager should
    ///   all be empty.
    ///
    /// # Return
    ///
    /// Returns a combinatorial map containing `n_darts + 1` darts, the amount of darts wanted plus
    /// the null dart (at index `NULL_DART_ID` i.e. `0`).
    ///
    /// # Example
    ///
    /// See [`CMap2`] example.
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
            unused_darts: UnusedDarts::new(n_darts),
            betas: BetaFunctions::new(n_darts),
            n_darts: n_darts + 1,
        }
    }
}
