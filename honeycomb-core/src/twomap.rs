//! Map objects
//!
//! This module contains code for the two main structures provided
//! by the crate:
//!
//! - [CMap2], a 2D combinatorial map implementation
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ IMPORTS

use crate::{
    AttrSparseVec, AttributeUpdate, CoordsFloat, DartIdentifier, EdgeCollection, EdgeIdentifier,
    FaceCollection, FaceIdentifier, Orbit2, OrbitPolicy, Vertex2, VertexCollection,
    VertexIdentifier, NULL_DART_ID,
};

use std::collections::BTreeSet;
#[cfg(feature = "utils")]
use std::{fs::File, io::Write};

// ------ CONTENT

/// Error-modeling enum
///
/// This enum is used to describe all non-panic errors that can occur when operating on a map.
#[derive(Debug, PartialEq)]
pub enum CMapError {
    /// Variant used when requesting a vertex using an ID that has no associated vertex
    /// in storage.
    UndefinedVertex,
}

// --- 2-MAP

const CMAP2_BETA: usize = 3;

/// Main map object.
///
/// Structure used to model 2D combinatorial map. The structure implements
/// basic operations:
///
/// - free dart addition/insertion/removal
/// - i-sewing/unsewing
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
/// - `vertices: Vec<Vertex>` -- List of vertices making up the represented mesh
/// - `free_darts: BTreeSet<DartIdentifier>` -- Set of free darts identifiers, i.e. empty
///   spots in the current dart list
/// - `betas: Vec<[DartIdentifier; 3]>` -- Array representation of the beta functions
/// - `n_darts: usize` -- Current number of darts (including the null dart)
///
/// Note that we encode *β<sub>0</sub>* as the inverse function of *β<sub>1</sub>*.
/// This is extremely useful (read *required*) to implement correct and efficient
/// i-cell computation. Additionally, while *β<sub>0</sub>* can be accessed using
/// the [Self::beta] method, we do not define 0-sew or 0-unsew operations.
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
/// ![CMAP2_EXAMPLE](../../images/CMap2Example.svg)
///
/// Note that the map we operate on has no boundaries. In addition to the different
/// operations realized at each step, we insert a few assertions to demonstrate the
/// progressive changes applied to the structure.
///
/// ```
/// # use honeycomb_core::CMapError;
/// # fn main() -> Result<(), CMapError> {
///
/// use honeycomb_core::{CMap2, FloatType, Orbit2, OrbitPolicy, Vertex2};
///
/// // build a triangle
/// let mut map: CMap2<FloatType> = CMap2::new(3);
/// map.one_link(1, 2);
/// map.one_link(2, 3);
/// map.one_link(3, 1);
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
/// map.add_free_darts(3);
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
/// assert_eq!(
///     map.replace_vertex(2, Vertex2::from((1.0, 0.0))),
///     Ok(Vertex2::from((1.5, 0.0)))
/// );
/// assert_eq!(
///     map.replace_vertex(3, Vertex2::from((0.0, 1.0))),
///     Ok(Vertex2::from((0.0, 1.5)))
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
/// let mut value_iterator = vertices.identifiers.iter().map(|vertex_id| map.vertex(*vertex_id));
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((0.0, 0.0)))); // vertex ID 1
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((0.0, 1.0)))); // vertex ID 3
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((1.0, 0.0)))); // vertex ID 5
/// assert_eq!(value_iterator.next(), Some(Vertex2::from((1.0, 1.0)))); // vertex ID 6
///
/// # Ok(())
/// # }
/// ```
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct CMap2<T: CoordsFloat> {
    /// List of vertices making up the represented mesh
    vertices: AttrSparseVec<Vertex2<T>>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list
    unused_darts: BTreeSet<DartIdentifier>,
    /// Array representation of the beta functions
    betas: Vec<[DartIdentifier; CMAP2_BETA]>,
    /// Current number of darts
    n_darts: usize,
}

// --- constructor
impl<T: CoordsFloat> CMap2<T> {
    /// Creates a new 2D combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts composing the new combinatorial map.
    /// - `n_vertices: usize` -- Number of vertices in the represented mesh.
    ///
    /// # Return / Panic
    ///
    /// Returns a combinatorial map containing:
    /// - `n_darts + 1` darts, the amount of darts wanted plus the null dart (at index 0).
    /// - 3 beta functions, *β<sub>0</sub>* being defined as the inverse of *β<sub>1</sub>*.
    /// - Default embed data associated to each dart.
    /// - `n_vertices` that the user will have to manually define a link to darts.
    /// - An empty list of currently free darts. This may be used for dart creation.
    ///
    /// # Example
    ///
    /// See [CMap2] example.
    ///
    pub fn new(n_darts: usize) -> Self {
        Self {
            vertices: AttrSparseVec::new(n_darts + 1),
            unused_darts: BTreeSet::new(),
            betas: vec![[0; CMAP2_BETA]; n_darts + 1],
            n_darts: n_darts + 1,
        }
    }
}

// --- dart-related code
impl<T: CoordsFloat> CMap2<T> {
    // --- read

    /// Return information about the current number of darts.
    pub fn n_darts(&self) -> usize {
        self.n_darts
    }

    /// Return information about the current number of unused darts.
    pub fn n_unused_darts(&self) -> usize {
        self.unused_darts.len()
    }

    // --- edit

    /// Add a new free dart to the combinatorial map.
    ///
    /// The dart is i-free for all i and is pushed to the list of existing darts, effectively
    /// making its identifier equal to the total number of darts (post-push).
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the created dart to allow for direct operations.
    ///
    pub fn add_free_dart(&mut self) -> DartIdentifier {
        let new_id = self.n_darts as DartIdentifier;
        self.n_darts += 1;
        self.betas.push([0; CMAP2_BETA]);
        self.vertices.extend(1);
        new_id
    }

    /// Add multiple new free darts to the combinatorial map.
    ///
    /// All darts are i-free for all i and are pushed to the end of the list of existing darts.
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts to have.
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the first created dart to allow for direct operations. Darts are
    /// positioned on range `ID..ID+n_darts`.
    ///
    pub fn add_free_darts(&mut self, n_darts: usize) -> DartIdentifier {
        let new_id = self.n_darts as DartIdentifier;
        self.n_darts += n_darts;
        self.betas.extend((0..n_darts).map(|_| [0; CMAP2_BETA]));
        self.vertices.extend(n_darts);
        new_id
    }

    /// Insert a new free dart to the combinatorial map.
    ///
    /// The dart is i-free for all i and may be inserted into an unused spot in the existing dart
    /// list. If no free spots exist, it will be pushed to the end of the list.
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the created dart to allow for direct operations.
    ///
    pub fn insert_free_dart(&mut self) -> DartIdentifier {
        if let Some(new_id) = self.unused_darts.pop_first() {
            self.betas[new_id as usize] = [0; CMAP2_BETA];
            new_id
        } else {
            self.add_free_dart()
        }
    }

    /// Remove a free dart from the combinatorial map.
    ///
    /// The removed dart identifier is added to the list of free dart. This way of proceeding is
    /// necessary as the structure relies on darts indexing for encoding data, making reordering of
    /// any sort extremely costly.
    ///
    /// By keeping track of free spots in the dart arrays, we can prevent too much memory waste,
    /// although at the cost of locality of reference.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of the dart to remove.
    ///
    /// # Panic
    ///
    /// This method may panic if:
    ///
    /// - The dart is not *i*-free for all *i*.
    /// - The dart is already marked as unused (Refer to [Self::remove_vertex] documentation for
    ///   a detailed breakdown of this choice).
    ///
    pub fn remove_free_dart(&mut self, dart_id: DartIdentifier) {
        assert!(self.is_free(dart_id));
        assert!(self.unused_darts.insert(dart_id));
        // this should not be required if the map is not corrupt
        // or in the middle of a more complex operation
        let b0d = self.beta::<0>(dart_id);
        let b1d = self.beta::<1>(dart_id);
        let b2d = self.beta::<2>(dart_id);
        self.betas[b0d as usize][1] = 0 as DartIdentifier;
        self.betas[b1d as usize][0] = 0 as DartIdentifier;
        self.betas[b2d as usize][2] = 0 as DartIdentifier;
    }
}

// --- beta-related code
impl<T: CoordsFloat> CMap2<T> {
    // --- read

    /// Compute the value of the i-th beta function of a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should
    /// be 0, 1 or 2 for a 2D map.
    ///
    /// # Return / Panic
    ///
    /// Return the identifier of the dart *d* such that *d = β<sub>i</sub>(dart)*. If the returned
    /// value is the null dart (i.e. a dart ID equal to 0), this means that *dart* is i-free.
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    ///
    pub fn beta<const I: u8>(&self, dart_id: DartIdentifier) -> DartIdentifier {
        assert!(I < 3);
        self.betas[dart_id as usize][I as usize]
    }

    /// Compute the value of the i-th beta function of a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    /// - `i: u8` -- Index of the beta function. *i* should be 0, 1 or 2 for a 2D map.
    ///
    /// # Return / Panic
    ///
    /// Return the identifier of the dart *d* such that *d = β<sub>i</sub>(dart)*. If the returned
    /// value is the null dart (i.e. a dart ID equal to 0), this means that *dart* is i-free.
    ///
    /// The method will panic if *i* is not 0, 1 or 2.
    ///
    pub fn beta_runtime(&self, i: u8, dart_id: DartIdentifier) -> DartIdentifier {
        assert!(i < 3);
        match i {
            0 => self.beta::<0>(dart_id),
            1 => self.beta::<1>(dart_id),
            2 => self.beta::<2>(dart_id),
            _ => unreachable!(),
        }
    }

    /// Check if a given dart is i-free.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should be 0, 1 or 2 for a 2D map.
    ///
    /// # Return / Panic
    ///
    /// Return a boolean indicating if *dart* is i-free, i.e. *β<sub>i</sub>(dart) = NullDart*.
    ///
    /// The function will panic if *I* is not 0, 1 or 2.
    ///
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdentifier) -> bool {
        self.beta::<I>(dart_id) == NULL_DART_ID
    }

    /// Check if a given dart is i-free, for all i.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// # Return / Panic
    ///
    /// Return a boolean indicating if *dart* is 0-free, 1-free and 2-free.
    ///
    pub fn is_free(&self, dart_id: DartIdentifier) -> bool {
        self.beta::<0>(dart_id) == NULL_DART_ID
            && self.beta::<1>(dart_id) == NULL_DART_ID
            && self.beta::<2>(dart_id) == NULL_DART_ID
    }

    // --- edit

    /// Set the values of the beta functions of a dart.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `betas: [DartIdentifier; 3]` -- Value of the images as
    ///   *[β<sub>0</sub>(dart), β<sub>1</sub>(dart), β<sub>2</sub>(dart)]*
    ///
    #[cfg(feature = "utils")]
    pub fn set_betas(&mut self, dart_id: DartIdentifier, betas: [DartIdentifier; CMAP2_BETA]) {
        self.betas[dart_id as usize] = betas;
    }
}

// --- icell-related code
impl<T: CoordsFloat> CMap2<T> {
    /// Fetch vertex identifier associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// # Return / Panic
    ///
    /// Return the identifier of the associated vertex.
    ///
    /// Cells identifiers are defined as the smallest identifier among the darts that make up the
    /// cell. This definition has three interesting properties:
    ///
    /// - A given cell ID can be computed from any dart of the cell, i.e. all darts have an
    /// associated cell ID.
    /// - Cell IDs are not affected by the order of traversal of the map.
    /// - Because the ID is computed in real time, there is no need to store cell IDs and ensure
    /// that the storage is consistent / up to date.
    ///
    /// These properties come at the literal cost of the computation routine, which is:
    /// 1. a BFS to compute a given orbit
    /// 2. a minimum computation on the IDs composing the orbit
    ///
    pub fn vertex_id(&self, dart_id: DartIdentifier) -> VertexIdentifier {
        // unwraping the result is safe because the orbit is always non empty
        Orbit2::new(self, OrbitPolicy::Vertex, dart_id)
            .min()
            .unwrap() as VertexIdentifier
    }

    /// Fetch edge associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// # Return / Panic
    ///
    /// Return the identifier of the associated edge.
    ///
    /// Cells identifiers are defined as the smallest identifier among the darts that make up the
    /// cell. This definition has three interesting properties:
    ///
    /// - A given cell ID can be computed from any dart of the cell, i.e. all darts have an
    /// associated cell ID.
    /// - Cell IDs are not affected by the order of traversal of the map.
    /// - Because the ID is computed in real time, there is no need to store cell IDs and ensure
    /// that the storage is consistent / up to date.
    ///
    /// These properties come at the literal cost of the computation routine, which is:
    /// 1. a BFS to compute a given orbit
    /// 2. a minimum computation on the IDs composing the orbit
    ///
    pub fn edge_id(&self, dart_id: DartIdentifier) -> EdgeIdentifier {
        // unwraping the result is safe because the orbit is always non empty
        Orbit2::new(self, OrbitPolicy::Edge, dart_id).min().unwrap() as EdgeIdentifier
    }

    /// Fetch face associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// # Return / Panic
    ///
    /// Return the identifier of the associated face.
    ///
    /// Cells identifiers are defined as the smallest identifier among the darts that make up the
    /// cell. This definition has three interesting properties:
    ///
    /// - A given cell ID can be computed from any dart of the cell, i.e. all darts have an
    /// associated cell ID.
    /// - Cell IDs are not affected by the order of traversal of the map.
    /// - Because the ID is computed in real time, there is no need to store cell IDs and ensure
    /// that the storage is consistent / up to date.
    ///
    /// These properties come at the literal cost of the computation routine, which is:
    /// 1. a BFS to compute a given orbit
    /// 2. a minimum computation on the IDs composing the orbit
    ///
    pub fn face_id(&self, dart_id: DartIdentifier) -> FaceIdentifier {
        // unwraping the result is safe because the orbit is always non empty
        Orbit2::new(self, OrbitPolicy::Face, dart_id).min().unwrap() as FaceIdentifier
    }

    /// Return an [Orbit2] object that can be used to iterate over darts of an i-cell.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Dimension of the cell of interest. *I* should be 0 (vertex), 1 (edge) or
    /// 2 (face) for a 2D map.
    ///
    /// # Return / Panic
    ///
    /// Returns an [Orbit2] that can be iterated upon to retrieve all dart member of the cell. Note
    /// that **the dart passed as an argument is included as the first element of the returned
    /// orbit**.
    ///
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdentifier) -> Orbit2<T> {
        assert!(I < 3);
        match I {
            0 => Orbit2::new(self, OrbitPolicy::Vertex, dart_id),
            1 => Orbit2::new(self, OrbitPolicy::Edge, dart_id),
            2 => Orbit2::new(self, OrbitPolicy::Face, dart_id),
            _ => unreachable!(),
        }
    }

    /// Return a collection of all the map's vertices.
    ///
    /// # Return / Panic
    ///
    /// Return a [VertexCollection] object containing a list of vertex identifiers, whose validity
    /// is ensured through an implicit lifetime condition on the structure and original map.
    ///
    pub fn fetch_vertices(&self) -> VertexCollection<T> {
        let mut marked: BTreeSet<DartIdentifier> = BTreeSet::new();
        // using a set for cells & converting it later to avoid duplicated values
        // from incomplete cells until they are correctly supported by Orbit2
        let mut vertex_ids: BTreeSet<DartIdentifier> = BTreeSet::new();
        (1..self.n_darts as DartIdentifier)
            .filter(|dart_id| !self.unused_darts.contains(dart_id)) // only used darts
            .for_each(|dart_id| {
                // if we haven't seen this dart yet
                if marked.insert(dart_id) {
                    // because we iterate from dart 1 to n_darts,
                    // the first dart we encounter is the min of its orbit
                    vertex_ids.insert(dart_id as VertexIdentifier);
                    // mark its orbit
                    Orbit2::new(self, OrbitPolicy::Vertex, dart_id).for_each(|did| {
                        marked.insert(did);
                    });
                }
            });
        VertexCollection::new(self, vertex_ids)
    }

    /// Return a collection of all the map's edges.
    ///
    /// # Return / Panic
    ///
    /// Return an [EdgeCollection] object containing a list of edge identifiers, whose validity
    /// is ensured through an implicit lifetime condition on the structure and original map.
    ///
    pub fn fetch_edges(&self) -> EdgeCollection<T> {
        let mut marked: BTreeSet<DartIdentifier> = BTreeSet::new();
        marked.insert(NULL_DART_ID);
        // using a set for cells & converting it later to avoid duplicated values
        // from incomplete cells until they are correctly supported by Orbit2
        let mut edge_ids: BTreeSet<EdgeIdentifier> = BTreeSet::new();
        (1..self.n_darts as DartIdentifier)
            .filter(|dart_id| !self.unused_darts.contains(dart_id)) // only used darts
            .for_each(|dart_id| {
                // if we haven't seen this dart yet
                if marked.insert(dart_id) {
                    // because we iterate from dart 1 to n_darts,
                    // the first dart we encounter is the min of its orbit
                    edge_ids.insert(dart_id as EdgeIdentifier);
                    // mark its orbit
                    Orbit2::new(self, OrbitPolicy::Edge, dart_id).for_each(|did| {
                        marked.insert(did);
                    });
                }
            });
        EdgeCollection::new(self, edge_ids)
    }

    /// Return a collection of all the map's faces.
    ///
    /// # Return / Panic
    ///
    /// Return a [FaceCollection] object containing a list of face identifiers, whose validity
    /// is ensured through an implicit lifetime condition on the structure and original map.
    ///
    pub fn fetch_faces(&self) -> FaceCollection<T> {
        let mut marked: BTreeSet<DartIdentifier> = BTreeSet::new();
        // using a set for cells & converting it later to avoid duplicated values
        // from incomplete cells until they are correctly supported by Orbit2
        let mut face_ids: BTreeSet<FaceIdentifier> = BTreeSet::new();
        (1..self.n_darts as DartIdentifier)
            .filter(|dart_id| !self.unused_darts.contains(dart_id)) // only used darts
            .for_each(|dart_id| {
                // if we haven't seen this dart yet
                if marked.insert(dart_id) {
                    // because we iterate from dart 1 to n_darts,
                    // the first dart we encounter is the min of its orbit
                    face_ids.insert(dart_id as FaceIdentifier);
                    // mark its orbit
                    Orbit2::new(self, OrbitPolicy::Face, dart_id).for_each(|did| {
                        marked.insert(did);
                    });
                }
            });
        FaceCollection::new(self, face_ids)
    }
}

// --- (un)sew operations
impl<T: CoordsFloat> CMap2<T> {
    /// 1-sew operation.
    ///
    /// This operation corresponds to *coherently linking* two darts via
    /// the *β<sub>1</sub>* function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    /// - `policy: SewPolicy` -- Geometrical sewing policy to follow.
    ///
    /// After the sewing operation, these darts will verify
    /// *β<sub>1</sub>(lhs_dart) = rhs_dart*. The *β<sub>0</sub>*
    /// function is also updated.
    ///
    /// # Return / Panic
    ///
    /// The method may panic if the two darts are not 1-sewable.
    ///
    pub fn one_sew(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        // this operation only makes sense if lhs_dart is associated to a fully defined edge, i.e.
        // its image through beta2 is defined & has a valid associated vertex (we assume the second
        // condition is valid if the first one is)
        // if that is not the case, the sewing operation becomes a linking operation
        let b2lhs_dart_id = self.beta::<2>(lhs_dart_id);
        if b2lhs_dart_id != NULL_DART_ID {
            let b2lhs_vid_old = self.vertex_id(b2lhs_dart_id);
            let rhs_vid_old = self.vertex_id(rhs_dart_id);
            let tmp = (
                self.vertices.remove(b2lhs_vid_old),
                self.vertices.remove(rhs_vid_old),
            );
            let new_vertex = match tmp {
                (Some(val1), Some(val2)) => Vertex2::merge(val1, val2),
                (Some(val), None) => Vertex2::merge_undefined(Some(val)),
                (None, Some(val)) => Vertex2::merge_undefined(Some(val)),
                (None, None) => Vertex2::merge_undefined(None),
            };
            // use b2lhs_vid as the index for the new vertex
            self.one_link(lhs_dart_id, rhs_dart_id);
            self.insert_vertex(self.vertex_id(rhs_dart_id), new_vertex);
        } else {
            self.one_link(lhs_dart_id, rhs_dart_id);
        }
    }

    /// 2-sew operation.
    ///
    /// This operation corresponds to *coherently linking* two darts via
    /// the *β<sub>2</sub>* function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    /// - `policy: SewPolicy` -- Geometrical sewing policy to follow.
    ///
    /// After the sewing operation, these darts will verify
    /// *β<sub>2</sub>(lhs_dart) = rhs_dart* and *β<sub>2</sub>(rhs_dart) = lhs_dart*.
    ///
    /// # Return / Panic
    ///
    /// The method may panic if:
    /// - the two darts are not 2-sewable,
    /// - the method cannot resolve orientation issues.
    ///
    pub fn two_sew(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        let b1lhs_dart_id = self.beta::<1>(lhs_dart_id);
        let b1rhs_dart_id = self.beta::<1>(rhs_dart_id);
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => self.two_link(lhs_dart_id, rhs_dart_id),
            // update vertex associated to b1rhs/lhs
            (true, false) => {
                // read current values / remove old ones
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let b1rhs_vid_old = self.vertex_id(b1rhs_dart_id);
                let tmp = (
                    self.vertices.remove(lhs_vid_old),
                    self.vertices.remove(b1rhs_vid_old),
                );
                let new_vertex = match tmp {
                    (Some(val1), Some(val2)) => Vertex2::merge(val1, val2),
                    (Some(val), None) => Vertex2::merge_undefined(Some(val)),
                    (None, Some(val)) => Vertex2::merge_undefined(Some(val)),
                    (None, None) => Vertex2::merge_undefined(None),
                };
                // update the topology (this is why we need the above lines)
                self.two_link(lhs_dart_id, rhs_dart_id);
                // reinsert correct value
                self.insert_vertex(self.vertex_id(lhs_dart_id), new_vertex);
            }
            // update vertex associated to b1lhs/rhs
            (false, true) => {
                // read current values / remove old ones
                let b1lhs_vid_old = self.vertex_id(b1lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                let tmp = (
                    self.vertices.remove(b1lhs_vid_old),
                    self.vertices.remove(rhs_vid_old),
                );
                let new_vertex = match tmp {
                    (Some(val1), Some(val2)) => Vertex2::merge(val1, val2),
                    (Some(val), None) => Vertex2::merge_undefined(Some(val)),
                    (None, Some(val)) => Vertex2::merge_undefined(Some(val)),
                    (None, None) => Vertex2::merge_undefined(None),
                };
                // update the topology (this is why we need the above lines)
                self.two_link(lhs_dart_id, rhs_dart_id);
                // reinsert correct value
                self.insert_vertex(self.vertex_id(rhs_dart_id), new_vertex);
            }
            // update both vertices making up the edge
            (false, false) => {
                // read current values / remove old ones
                // (lhs/b1rhs) vertex
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let b1rhs_vid_old = self.vertex_id(b1rhs_dart_id);
                let tmpa = (
                    self.vertices.remove(lhs_vid_old),
                    self.vertices.remove(b1rhs_vid_old),
                );
                // (b1lhs/rhs) vertex
                let b1lhs_vid_old = self.vertex_id(b1lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                let tmpb = (
                    self.vertices.remove(b1lhs_vid_old),
                    self.vertices.remove(rhs_vid_old),
                );

                // check orientation
                #[rustfmt::skip]
                if let (
                    (Some(l_vertex), Some(b1r_vertex)),
                    (Some(b1l_vertex), Some(r_vertex)),
                ) = (tmpa, tmpb) {
                    let lhs_vector = b1l_vertex - l_vertex;
                    let rhs_vector = b1r_vertex - r_vertex;
                    // dot product should be negative if the two darts have opposite direction
                    // we could also put restriction on the angle made by the two darts to prevent
                    // drastic deformation
                    assert!(
                        lhs_vector.dot(&rhs_vector) < T::zero(),
                        "Dart {} and {} do not have consistent orientation for 2-sewing",
                        lhs_dart_id,
                        rhs_dart_id
                    );
                };

                // proceed with new vertices creation & insertion
                let new_vertexa = match tmpa {
                    (Some(val1), Some(val2)) => Vertex2::merge(val1, val2),
                    (Some(val), None) => Vertex2::merge_undefined(Some(val)),
                    (None, Some(val)) => Vertex2::merge_undefined(Some(val)),
                    (None, None) => Vertex2::merge_undefined(None),
                };

                let new_vertexb = match tmpb {
                    (Some(val1), Some(val2)) => Vertex2::merge(val1, val2),
                    (Some(val), None) => Vertex2::merge_undefined(Some(val)),
                    (None, Some(val)) => Vertex2::merge_undefined(Some(val)),
                    (None, None) => Vertex2::merge_undefined(None),
                };
                // update the topology
                self.two_link(lhs_dart_id, rhs_dart_id);

                // reinsert correct values
                self.insert_vertex(self.vertex_id(lhs_dart_id), new_vertexa);
                self.insert_vertex(self.vertex_id(rhs_dart_id), new_vertexb);
            }
        }
    }

    /// 1-unsew operation.
    ///
    /// This operation corresponds to *coherently separating* two darts linked
    /// via the *β<sub>1</sub>* function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to separate.
    /// - `policy: UnsewPolicy` -- Geometrical unsewing policy to follow.
    ///
    /// Note that we do not need to take two darts as arguments since the
    /// second dart can be obtained through the *β<sub>1</sub>* function. The
    /// *β<sub>0</sub>* function is also updated.
    ///
    pub fn one_unsew(&mut self, lhs_dart_id: DartIdentifier) {
        let b2lhs_dart_id = self.beta::<2>(lhs_dart_id);
        if b2lhs_dart_id != NULL_DART_ID {
            // read current values / remove old ones
            let rhs_dart_id = self.beta::<1>(lhs_dart_id);
            // we only need to remove a single vertex since we're unlinking
            let vertex = self.remove_vertex(self.vertex_id(rhs_dart_id)).unwrap();
            let (v1, v2) = Vertex2::split(vertex);
            // update the topology
            self.one_unlink(lhs_dart_id);
            // reinsert correct values
            let _ = self.replace_vertex(self.vertex_id(b2lhs_dart_id), v1);
            let _ = self.replace_vertex(self.vertex_id(rhs_dart_id), v2);
        } else {
            self.one_unlink(lhs_dart_id)
        }
    }

    /// 2-unsew operation.
    ///
    /// This operation corresponds to *coherently separating* two darts linked
    /// via the *β<sub>2</sub>* function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to separate.
    /// - `policy: UnsewPolicy` -- Geometrical unsewing policy to follow.
    ///
    /// Note that we do not need to take two darts as arguments since the
    /// second dart can be obtained through the *β<sub>2</sub>* function.
    ///
    pub fn two_unsew(&mut self, lhs_dart_id: DartIdentifier) {
        let rhs_dart_id = self.beta::<2>(lhs_dart_id);
        let b1lhs_dart_id = self.beta::<1>(lhs_dart_id);
        let b1rhs_dart_id = self.beta::<1>(rhs_dart_id);
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            (true, true) => self.two_unlink(lhs_dart_id),
            (true, false) => {
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                let rhs_vertex = self.remove_vertex(rhs_vid_old).unwrap();
                let (v1, v2) = Vertex2::split(rhs_vertex);
                self.two_unlink(lhs_dart_id);
                self.insert_vertex(self.vertex_id(b1lhs_dart_id), v1);
                self.insert_vertex(self.vertex_id(rhs_dart_id), v2);
            }
            (false, true) => {
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let lhs_vertex = self.remove_vertex(lhs_vid_old).unwrap();
                let (v1, v2) = Vertex2::split(lhs_vertex);
                self.two_unlink(lhs_dart_id);
                self.insert_vertex(self.vertex_id(lhs_dart_id), v1);
                self.insert_vertex(self.vertex_id(b1rhs_dart_id), v2);
            }
            (false, false) => {
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                let lhs_vertex = self.remove_vertex(lhs_vid_old).unwrap();
                let rhs_vertex = self.remove_vertex(rhs_vid_old).unwrap();
                self.two_unlink(lhs_dart_id);
                let (rhs_v1, rhs_v2) = Vertex2::split(rhs_vertex);
                let (lhs_v1, lhs_v2) = Vertex2::split(lhs_vertex);

                self.insert_vertex(self.vertex_id(b1lhs_dart_id), rhs_v1);
                self.insert_vertex(self.vertex_id(rhs_dart_id), rhs_v2);
                self.insert_vertex(self.vertex_id(lhs_dart_id), lhs_v1);
                self.insert_vertex(self.vertex_id(b1rhs_dart_id), lhs_v2);
            }
        }
    }
}

// --- (un)link operations
impl<T: CoordsFloat> CMap2<T> {
    /// 1-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>1</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s). The *β<sub>0</sub>* function is also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    pub fn one_link(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        // we could technically overwrite the value, but these assertions
        // makes it easier to assert algorithm correctness
        assert!(self.is_i_free::<1>(lhs_dart_id));
        assert!(self.is_i_free::<0>(rhs_dart_id));
        self.betas[lhs_dart_id as usize][1] = rhs_dart_id; // set beta_1(lhs_dart) to rhs_dart
        self.betas[rhs_dart_id as usize][0] = lhs_dart_id; // set beta_0(rhs_dart) to lhs_dart
    }

    /// 2-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>2</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    pub fn two_link(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        // we could technically overwrite the value, but these assertions
        // make it easier to assert algorithm correctness
        assert!(self.is_i_free::<2>(lhs_dart_id));
        assert!(self.is_i_free::<2>(rhs_dart_id));
        self.betas[lhs_dart_id as usize][2] = rhs_dart_id; // set beta_2(lhs_dart) to rhs_dart
        self.betas[rhs_dart_id as usize][2] = lhs_dart_id; // set beta_2(rhs_dart) to lhs_dart
    }

    /// 1-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>1</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s). The *β<sub>0</sub>* function is
    /// also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    pub fn one_unlink(&mut self, lhs_dart_id: DartIdentifier) {
        let rhs_dart_id = self.beta::<1>(lhs_dart_id); // fetch id of beta_1(lhs_dart)
        self.betas[lhs_dart_id as usize][1] = 0; // set beta_1(lhs_dart) to NullDart
        self.betas[rhs_dart_id as usize][0] = 0; // set beta_0(rhs_dart) to NullDart
    }

    /// 2-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>2</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    pub fn two_unlink(&mut self, lhs_dart_id: DartIdentifier) {
        let rhs_dart_id = self.beta::<2>(lhs_dart_id); // fetch id of beta_2(lhs_dart)
        self.betas[lhs_dart_id as usize][2] = 0; // set beta_2(dart) to NullDart
        self.betas[rhs_dart_id as usize][2] = 0; // set beta_2(beta_2(dart)) to NullDart
    }
}

// --- vertex attributes
// this should eventually be replaced by a generalized structure to handle
// different kind of attributes for all the i-cells.
impl<T: CoordsFloat> CMap2<T> {
    /// Return the current number of vertices.
    pub fn n_vertices(&self) -> usize {
        self.vertices.n_attributes()
    }

    /// Fetch vertex value associated to a given identifier.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the given vertex.
    ///
    /// # Return / Panic
    ///
    /// Return a reference to the [Vertex2] associated to the ID.
    ///
    pub fn vertex(&self, vertex_id: VertexIdentifier) -> Vertex2<T> {
        self.vertices.get(vertex_id).unwrap()
    }

    /// Insert a vertex in the combinatorial map.
    ///
    /// This method can be interpreted as giving a value to the vertex of a specific ID. Vertices
    /// implicitly exist through topology, but their spatial representation is not automatically
    /// created at first.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Vertex identifier to attribute a value to.
    /// - `vertex: impl Into<Vertex2>` -- Value used to create a [Vertex2] value.
    ///
    /// # Return
    ///
    /// Return an option which may contain the previous value associated to the specified vertex ID.
    ///
    pub fn insert_vertex(&mut self, vertex_id: VertexIdentifier, vertex: impl Into<Vertex2<T>>) {
        self.vertices.insert(vertex_id, vertex.into())
    }

    /// Remove a vertex from the combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to remove.
    ///
    /// # Return
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(v: Vertex2)` -- The vertex was successfully removed & its value was returned
    /// - `Err(CMapError::UndefinedVertexID)` -- The vertex was not found in the internal storage
    ///
    pub fn remove_vertex(&mut self, vertex_id: VertexIdentifier) -> Result<Vertex2<T>, CMapError> {
        if let Some(val) = self.vertices.remove(vertex_id) {
            return Ok(val);
        }
        Err(CMapError::UndefinedVertex)
    }

    /// Try to overwrite the given vertex with a new value.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to replace.
    /// - `vertex: impl<Into<Vertex2>>` -- New value for the vertex.
    ///
    /// # Return / Panic
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(v: Vertex2)` -- The vertex was successfully overwritten & its previous value was
    /// returned
    /// - `Err(CMapError::UnknownVertexID)` -- The vertex was not found in the internal storage
    ///
    pub fn replace_vertex(
        &mut self,
        vertex_id: VertexIdentifier,
        vertex: impl Into<Vertex2<T>>,
    ) -> Result<Vertex2<T>, CMapError> {
        if let Some(val) = self.vertices.replace(vertex_id, vertex.into()) {
            return Ok(val);
        };
        Err(CMapError::UndefinedVertex)
    }
}

#[cfg(any(doc, feature = "utils"))]
impl<T: CoordsFloat> CMap2<T> {
    /// Computes the total allocated space dedicated to the map.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `rootname: &str` -- root of the filename used to save results.
    ///
    /// # Return / Panic
    ///
    /// The results of this method is saved as a csv file named `<rootname>_allocated.csv`.
    /// The csv file is structured as follows:
    ///
    /// ```text
    /// key, memory (bytes)
    /// cat1_member1, val
    /// cat1_member2, val
    /// cat1_total, val
    /// cat2_member1, val
    /// cat2_member2, val
    /// cat2_member3, val
    /// cat2_total, val
    /// cat3_member1, val
    /// cat3_total, val
    /// ```
    ///
    /// It is mainly designed to be used in dedicated scripts for plotting & analysis.
    ///
    /// The metod may panic if, for any reason, it is unable to write to the file.
    ///
    /// # Example
    ///
    /// An example going over all three `size` methods is provided in the `honeycomb-utils`
    /// crate. You can run it using the following command:
    ///
    /// ```shell
    /// cargo run --example memory_usage
    /// ```
    ///
    /// The output data can be visualized using the `memory_usage.py` script.
    ///
    pub fn allocated_size(&self, rootname: &str) {
        let mut file = File::create(rootname.to_owned() + "_allocated.csv").unwrap();
        writeln!(file, "key, memory (bytes)").unwrap();

        // beta
        let mut beta_total = 0;
        (0..3).for_each(|beta_id| {
            let mem = self.betas.capacity() * std::mem::size_of::<DartIdentifier>();
            writeln!(file, "beta_{beta_id}, {mem}").unwrap();
            beta_total += mem;
        });
        writeln!(file, "beta_total, {beta_total}").unwrap();

        // cells
        // using 2 * sizeof(f64) bc sizeof(array) always is the size of a pointer
        let geometry_vertex = self.vertices.allocated_size();
        let geometry_total = geometry_vertex;
        writeln!(file, "geometry_vertex, {geometry_vertex}").unwrap();
        writeln!(file, "geometry_total, {geometry_total}").unwrap();

        // others
        let others_freedarts = self.unused_darts.len();
        let others_counters = 2 * std::mem::size_of::<usize>();
        let others_total = others_freedarts + others_counters;
        writeln!(file, "others_freedarts, {others_freedarts}").unwrap();
        writeln!(file, "others_counters, {others_counters}").unwrap();
        writeln!(file, "others_total, {others_total}").unwrap();
    }

    /// Computes the total used space dedicated to the map.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `rootname: &str` -- root of the filename used to save results.
    ///
    /// # Return / Panic
    ///
    /// The results of this method is saved as a csv file named `<rootname>_allocated.csv`.
    /// The csv file is structured as follows:
    ///
    /// ```text
    /// key, memory (bytes)
    /// cat1_member1, val
    /// cat1_member2, val
    /// cat1_total, val
    /// cat2_member1, val
    /// cat2_member2, val
    /// cat2_member3, val
    /// cat2_total, val
    /// cat3_member1, val
    /// cat3_total, val
    /// ```
    ///
    /// It is mainly designed to be used in dedicated scripts for plotting & analysis.
    ///
    /// The metod may panic if, for any reason, it is unable to write to the file.
    ///
    /// # Example
    ///
    /// An example going over all three `size` methods is provided in the `honeycomb-utils`
    /// crate. You can run it using the following command:
    ///
    /// ```shell
    /// cargo run --example memory_usage
    /// ```
    ///
    /// The output data can be visualized using the `memory_usage.py` script.
    ///
    pub fn effective_size(&self, rootname: &str) {
        let mut file = File::create(rootname.to_owned() + "_effective.csv").unwrap();
        writeln!(file, "key, memory (bytes)").unwrap();

        // beta
        let mut beta_total = 0;
        (0..3).for_each(|beta_id| {
            let mem = self.n_darts * std::mem::size_of::<DartIdentifier>();
            writeln!(file, "beta_{beta_id}, {mem}").unwrap();
            beta_total += mem;
        });
        writeln!(file, "beta_total, {beta_total}").unwrap();

        // embed
        let embed_vertex = self.n_darts * std::mem::size_of::<VertexIdentifier>();
        let embed_face = self.n_darts * std::mem::size_of::<FaceIdentifier>();
        let embed_total = embed_vertex + embed_face;
        writeln!(file, "embed_vertex, {embed_vertex}").unwrap();
        writeln!(file, "embed_face, {embed_face}").unwrap();
        writeln!(file, "embed_total, {embed_total}").unwrap();

        // cells
        // using 2 * sizeof(f64) bc sizeof(array) always is the size of a pointer
        let geometry_vertex = self.vertices.effective_size();
        let geometry_total = geometry_vertex;
        writeln!(file, "geometry_vertex, {geometry_vertex}").unwrap();
        writeln!(file, "geometry_total, {geometry_total}").unwrap();

        // others
        let others_freedarts = self.unused_darts.len();
        let others_counters = 2 * std::mem::size_of::<usize>();
        let others_total = others_freedarts + others_counters;
        writeln!(file, "others_freedarts, {others_freedarts}").unwrap();
        writeln!(file, "others_counters, {others_counters}").unwrap();
        writeln!(file, "others_total, {others_total}").unwrap();
    }

    /// Computes the actual used space dedicated to the map.
    ///
    /// *Actual used space* refers to the total used space minus empty spots
    /// in the structure.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `rootname: &str` -- root of the filename used to save results.
    ///
    /// # Return / Panic
    ///
    /// The results of this method is saved as a csv file named `<rootname>_allocated.csv`.
    /// The csv file is structured as follows:
    ///
    /// ```text
    /// key, memory (bytes)
    /// cat1_member1, val
    /// cat1_member2, val
    /// cat1_total, val
    /// cat2_member1, val
    /// cat2_member2, val
    /// cat2_member3, val
    /// cat2_total, val
    /// cat3_member1, val
    /// cat3_total, val
    /// ```
    ///
    /// It is mainly designed to be used in dedicated scripts for plotting & analysis.
    ///
    /// The metod may panic if, for any reason, it is unable to write to the file.
    ///
    /// # Example
    ///
    /// An example going over all three `size` methods is provided in the `honeycomb-utils`
    /// crate. You can run it using the following command:
    ///
    /// ```shell
    /// cargo run --example memory_usage
    /// ```
    ///
    /// The output data can be visualized using the `memory_usage.py` script.
    ///
    pub fn used_size(&self, rootname: &str) {
        let mut file = File::create(rootname.to_owned() + "_used.csv").unwrap();
        writeln!(file, "key, memory (bytes)").unwrap();

        let n_used_darts = self.n_darts - self.unused_darts.len();

        // beta
        let mut beta_total = 0;
        (0..3).for_each(|beta_id| {
            let mem = n_used_darts * std::mem::size_of::<DartIdentifier>();
            writeln!(file, "beta_{beta_id}, {mem}").unwrap();
            beta_total += mem;
        });
        writeln!(file, "beta_total, {beta_total}").unwrap();

        // embed
        let embed_vertex = n_used_darts * std::mem::size_of::<VertexIdentifier>();
        let embed_face = n_used_darts * std::mem::size_of::<FaceIdentifier>();
        let embed_total = embed_vertex + embed_face;
        writeln!(file, "embed_vertex, {embed_vertex}").unwrap();
        writeln!(file, "embed_face, {embed_face}").unwrap();
        writeln!(file, "embed_total, {embed_total}").unwrap();

        // cells
        // using 2 * sizeof(f64) bc sizeof(array) always is the size of a pointer
        let geometry_vertex = self.vertices.used_size();
        let geometry_total = geometry_vertex;
        writeln!(file, "geometry_vertex, {geometry_vertex}").unwrap();
        writeln!(file, "geometry_total, {geometry_total}").unwrap();

        // others
        let others_freedarts = self.unused_darts.len();
        let others_counters = 2 * std::mem::size_of::<usize>();
        let others_total = others_freedarts + others_counters;
        writeln!(file, "others_freedarts, {others_freedarts}").unwrap();
        writeln!(file, "others_counters, {others_counters}").unwrap();
        writeln!(file, "others_total, {others_total}").unwrap();
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use crate::{CMap2, FloatType, Orbit2, OrbitPolicy, Vertex2};

    #[test]
    fn example_test() {
        // build a triangle
        let mut map: CMap2<FloatType> = CMap2::new(3);
        map.one_link(1, 2);
        map.one_link(2, 3);
        map.one_link(3, 1);
        map.insert_vertex(1, (0.0, 0.0));
        map.insert_vertex(2, (1.0, 0.0));
        map.insert_vertex(3, (0.0, 1.0));

        // checks
        let faces = map.fetch_faces();
        assert_eq!(faces.identifiers.len(), 1);
        assert_eq!(faces.identifiers[0], 1);
        let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
        assert_eq!(face.next(), Some(1));
        assert_eq!(face.next(), Some(2));
        assert_eq!(face.next(), Some(3));
        assert_eq!(face.next(), None);

        // build a second triangle
        map.add_free_darts(3);
        map.one_link(4, 5);
        map.one_link(5, 6);
        map.one_link(6, 4);
        map.insert_vertex(4, (0.0, 2.0));
        map.insert_vertex(5, (2.0, 0.0));
        map.insert_vertex(6, (1.0, 1.0));

        // checks
        let faces = map.fetch_faces();
        assert_eq!(&faces.identifiers, &[1, 4]);
        let mut face = Orbit2::new(&map, OrbitPolicy::Face, 4);
        assert_eq!(face.next(), Some(4));
        assert_eq!(face.next(), Some(5));
        assert_eq!(face.next(), Some(6));
        assert_eq!(face.next(), None);

        // sew both triangles
        map.two_sew(2, 4);

        // checks
        assert_eq!(map.beta::<2>(2), 4);
        assert_eq!(map.vertex_id(2), 2);
        assert_eq!(map.vertex_id(5), 2);
        assert_eq!(map.vertex(2), Vertex2::from((1.5, 0.0)));
        assert_eq!(map.vertex_id(3), 3);
        assert_eq!(map.vertex_id(4), 3);
        assert_eq!(map.vertex(3), Vertex2::from((0.0, 1.5)));
        let edges = map.fetch_edges();
        assert_eq!(&edges.identifiers, &[1, 2, 3, 5, 6]);

        // adjust bottom-right & top-left vertex position
        assert_eq!(
            map.replace_vertex(2, Vertex2::from((1.0, 0.0))),
            Ok(Vertex2::from((1.5, 0.0)))
        );
        assert_eq!(map.vertex(2), Vertex2::from((1.0, 0.0)));
        assert_eq!(
            map.replace_vertex(3, Vertex2::from((0.0, 1.0))),
            Ok(Vertex2::from((0.0, 1.5)))
        );
        assert_eq!(map.vertex(3), Vertex2::from((0.0, 1.0)));

        // separate the diagonal from the rest
        map.one_unsew(1);
        map.one_unsew(2);
        map.one_unsew(6);
        map.one_unsew(4);
        // break up & remove the diagonal
        map.two_unsew(2); // this makes dart 2 and 4 free
        map.remove_free_dart(2);
        map.remove_free_dart(4);
        // sew the square back up
        map.one_sew(1, 5);
        map.one_sew(6, 3);

        // i-cells
        let faces = map.fetch_faces();
        assert_eq!(&faces.identifiers, &[1]);
        let edges = map.fetch_edges();
        assert_eq!(&edges.identifiers, &[1, 3, 5, 6]);
        let vertices = map.fetch_vertices();
        assert_eq!(&vertices.identifiers, &[1, 3, 5, 6]);
        assert_eq!(map.vertex(1), Vertex2::from((0.0, 0.0)));
        assert_eq!(map.vertex(5), Vertex2::from((1.0, 0.0)));
        assert_eq!(map.vertex(6), Vertex2::from((1.0, 1.0)));
        assert_eq!(map.vertex(3), Vertex2::from((0.0, 1.0)));
        // darts
        assert_eq!(map.n_unused_darts(), 2); // there are unused darts since we removed the diagonal
        assert_eq!(map.beta_runtime(1, 1), 5);
        assert_eq!(map.beta_runtime(1, 5), 6);
        assert_eq!(map.beta_runtime(1, 6), 3);
        assert_eq!(map.beta_runtime(1, 3), 1);
    }

    #[test]
    #[should_panic]
    fn remove_vertex_twice() {
        // in its default state, all darts/vertices of a map are considered to be used
        let mut map: CMap2<FloatType> = CMap2::new(4);
        // set vertex 1 as unused
        map.remove_vertex(1).unwrap();
        // set vertex 1 as unused, again
        map.remove_vertex(1).unwrap(); // this should panic
    }

    #[test]
    #[should_panic]
    fn remove_dart_twice() {
        // in its default state, all darts/vertices of a map are considered to be used
        // darts are also free
        let mut map: CMap2<FloatType> = CMap2::new(4);
        // set dart 1 as unused
        map.remove_free_dart(1);
        // set dart 1 as unused, again
        map.remove_free_dart(1); // this should panic
    }
}
