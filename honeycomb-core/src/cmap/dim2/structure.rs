//! Main definitions
//!
//! This module contains the main structure definition ([`CMap2`]) as well as its constructor
//! implementation.

use crate::attributes::{AttrSparseVec, AttrStorageManager, UnknownAttributeStorage};
use crate::cmap::components::{betas::BetaFunctions, unused::UnusedDarts};
use crate::geometry::{CoordsFloat, Vertex2};

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
/// - we create the map using its builder structure: [`CMapBuilder`][crate::prelude::CMapBuilder]
/// - we insert a few assertions to demonstrate the progressive changes applied to the structure
/// - even though the faces are represented in the figure, they are not stored in the structure
/// - we use a lot of methods with the `force_` prefix; these are convenience methods when
///   synchronization isn't needed
///
/// ```
/// # fn main() {
/// use honeycomb_core::{
///     cmap::{CMap2, CMapBuilder, Orbit2, OrbitPolicy},
///     geometry::Vertex2
/// };
///
/// // build a triangle (A)
/// let mut map: CMap2<f64> = CMapBuilder::default().n_darts(3).build().unwrap(); // three darts
/// map.force_link::<1>(1, 2); // beta1(1) = 2 & beta0(2) = 1
/// map.force_link::<1>(2, 3); // beta1(2) = 3 & beta0(3) = 2
/// map.force_link::<1>(3, 1); // beta1(3) = 1 & beta0(1) = 3
/// map.force_write_vertex(1, (0.0, 0.0));
/// map.force_write_vertex(2, (1.0, 0.0));
/// map.force_write_vertex(3, (0.0, 1.0));
///
/// // we can go through the face using an orbit
/// let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
/// assert_eq!(face.next(), Some(1));
/// assert_eq!(face.next(), Some(2));
/// assert_eq!(face.next(), Some(3));
/// assert_eq!(face.next(), None);
///
/// // build a second triangle (B)
/// let first_added_dart_id = map.add_free_darts(3);
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
/// map.remove_free_dart(2);
/// map.remove_free_dart(4);
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
