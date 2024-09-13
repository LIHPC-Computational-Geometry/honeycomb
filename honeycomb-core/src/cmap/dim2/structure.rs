//! Main definitions
//!
//! This module contains the main structure definition ([`CMap2`]) as well as its constructor
//! implementation.

// ------ IMPORTS

use super::CMAP2_BETA;
use crate::prelude::{DartIdentifier, Vertex2};
use crate::{
    attributes::{AttrSparseVec, AttrStorageManager, UnknownAttributeStorage},
    geometry::CoordsFloat,
};
use std::collections::BTreeSet;

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
///
/// ```
/// # fn main() {
///
/// use honeycomb_core::prelude::{CMap2, CMapBuilder, Orbit2, OrbitPolicy, Vertex2};
///
/// // build a triangle
/// let mut map: CMap2<f64> = CMapBuilder::default().n_darts(3).build().unwrap(); // three darts
/// map.one_link(1, 2); // beta1(1) = 2 & beta0(2) = 1
/// map.one_link(2, 3); // beta1(2) = 3 & beta0(3) = 2
/// map.one_link(3, 1); // beta1(3) = 1 & beta0(1) = 3
/// map.insert_vertex(1, (0.0, 0.0));
/// map.insert_vertex(2, (1.0, 0.0));
/// map.insert_vertex(3, (0.0, 1.0));
///
/// // we can go through the face using an orbit
/// let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
/// assert_eq!(face.next(), Some(1));
/// assert_eq!(face.next(), Some(2));
/// assert_eq!(face.next(), Some(3));
/// assert_eq!(face.next(), None);
///
/// // build a second triangle
/// let first_added_dart_id = map.add_free_darts(3);
/// assert_eq!(first_added_dart_id, 4);
/// map.one_link(4, 5);
/// map.one_link(5, 6);
/// map.one_link(6, 4);
/// map.insert_vertex(4, (0.0, 2.0));
/// map.insert_vertex(5, (2.0, 0.0));
/// map.insert_vertex(6, (1.0, 1.0));
///
/// // there should be two faces now
/// let faces = map.fetch_faces();
/// assert_eq!(&faces.identifiers, &[1, 4]);
///
/// // sew both triangles
/// map.two_sew(2, 4);
///
/// // there are 5 edges now, making up a square & its diagonal
/// let edges = map.fetch_edges();
/// assert_eq!(&edges.identifiers, &[1, 2, 3, 5, 6]);
///
/// // adjust bottom-right & top-left vertex position
/// // the returned values were the average of the sewn vertices
/// assert_eq!(
///     map.replace_vertex(2, Vertex2::from((1.0, 0.0))),
///     Some(Vertex2(1.5, 0.0))
/// );
/// assert_eq!(
///     map.replace_vertex(3, Vertex2::from((0.0, 1.0))),
///     Some(Vertex2(0.0, 1.5))
/// );
///
/// // separate the diagonal from the rest
/// map.one_unsew(1);
/// map.one_unsew(2);
/// map.one_unsew(6);
/// map.one_unsew(4);
/// // break up & remove the diagonal
/// map.two_unsew(2); // this makes dart 2 and 4 free
/// map.remove_free_dart(2);
/// map.remove_free_dart(4);
/// // sew the square back up
/// map.one_sew(1, 5);
/// map.one_sew(6, 3);
///
/// // there's only the square face left
/// let faces = map.fetch_faces();
/// assert_eq!(&faces.identifiers, &[1]);
/// // we can check the vertices
/// let vertices = map.fetch_vertices();
/// let mut value_iterator = vertices.identifiers.iter().map(|vertex_id| map.vertex(*vertex_id).unwrap());
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((0.0, 0.0)))); // vertex ID 1
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((0.0, 1.0)))); // vertex ID 3
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((1.0, 0.0)))); // vertex ID 5
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((1.0, 1.0)))); // vertex ID 6
///
/// # }
/// ```
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct CMap2<T: CoordsFloat> {
    /// List of vertices making up the represented mesh
    pub(super) attributes: AttrStorageManager,
    /// List of vertices making up the represented mesh
    pub(super) vertices: AttrSparseVec<Vertex2<T>>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list
    pub(super) unused_darts: BTreeSet<DartIdentifier>,
    /// Array representation of the beta functions
    pub(super) betas: Vec<[DartIdentifier; CMAP2_BETA]>,
    /// Current number of darts
    pub(super) n_darts: usize,
}

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
            unused_darts: BTreeSet::new(),
            betas: vec![[0; CMAP2_BETA]; n_darts + 1],
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
            unused_darts: BTreeSet::new(),
            betas: vec![[0; CMAP2_BETA]; n_darts + 1],
            n_darts: n_darts + 1,
        }
    }
}
