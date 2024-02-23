//! Map objects
//!
//! This module contains code for the two main structures provided
//! by the crate:
//! - [TwoMap], a 2D combinatorial map implementation
//! -
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

// ------ IMPORTS

use crate::{
    DartIdentifier, FaceIdentifier, SewPolicy, UnsewPolicy, VertexIdentifier, NULL_DART_ID,
};

use super::{
    dart::{CellIdentifiers, DartData},
    embed::{Face, Vertex2},
};

// ------ CONTENT

#[derive(Debug)]
pub enum MapError {
    VertexOOB,
}

// --- 2-MAP

const TWO_MAP_BETA: usize = 3;

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
/// # Fields
///
/// Fields are kept private in order to better define interfaces. The structure
/// contains the following data:
///
/// - `vertices: Vec<Vertex>` -- List of vertices making up the represented mesh
/// - `free_vertices: Vec<VertexIdentifier>` -- List of free vertex identifiers,
///   i.e. empty spots in the current vertex list
/// - `faces: Vec<Face>` -- List of faces making up the represented mesh
/// - `dart_data: DartData<N_MARKS>` -- List of embedded data associated with darts
/// - `free_darts: Vec<DartIdentifier>` -- List of free darts identifiers, i.e. empty
///   spots in the current dart list
/// - `betas: Vec<[DartIdentifier; 3]>` -- Array representation of the beta functions
/// - `n_darts: usize` -- Current number of darts (including the null dart)
/// - `n_vertices: usize` -- Current number of vertices
///
/// Note that we encode *β<sub>0</sub>* as the inverse function of *β<sub>1</sub>*.
/// This is extremely useful (read *required*) to implement correct and efficient
/// i-cell computation. Additionally, while *β<sub>0</sub>* can be accessed using
/// the [Self::beta] method, we do not define 0-sew or 0-unsew operations.
///
/// # Generics
///
/// - `const N_MARKS: usize` -- Number of marks used for search algorithms.
///   This corresponds to the number of search that can be done concurrently.
///
/// # Example
///
/// The following example goes over multiple operations on the mesh in order
/// to demonstrate general usage of the structure and its methods.
///
/// ![TWOMAP_EXAMPLE](link_to_example_file.svg)
///
/// Note that the map we operate on has no boundaries. In addition to the different
/// operations realized at each step, we insert a few assertions to demonstrate the
/// progressive changes applied to the structure.
///
/// ```
/// use honeycomb_core::{
///     DartIdentifier, SewPolicy, TwoMap, UnsewPolicy, VertexIdentifier, NULL_DART_ID,
/// };
///
/// // --- Map creation
///
/// // create a map with 3 non-null darts & 3 vertices
/// let mut map: TwoMap<1> = TwoMap::new(3, 3);
///
/// // the two following lines are not strictly necessary, you may use integers directly
/// let (d1, d2, d3): (DartIdentifier, DartIdentifier, DartIdentifier) = (1, 2, 3);
/// let (v1, v2, v3): (VertexIdentifier, VertexIdentifier, VertexIdentifier) = (0, 1, 2);
///
/// // place the vertices in space
/// map.set_vertex(v1, [0.0, 0.0]);
/// map.set_vertex(v2, [0.0, 10.0]);
/// map.set_vertex(v3, [10.0, 0.0]);
/// // associate dart to vertices
/// map.set_vertexid(d1, v1);
/// map.set_vertexid(d2, v2);
/// map.set_vertexid(d3, v3);
/// // define beta values to form a face
/// map.set_betas(d1, [d3, d2, NULL_DART_ID]); // beta 0 / 1 / 2 (d1) = d3 / d2 / null
/// map.set_betas(d2, [d1, d3, NULL_DART_ID]); // beta 0 / 1 / 2 (d2) = d1 / d3 / null
/// map.set_betas(d3, [d2, d1, NULL_DART_ID]); // beta 0 / 1 / 2 (d3) = d2 / d1 / null
///
/// // build the face we just linked & fetch the id for checks
/// let fface_id = map.build_face(d1);
///
/// // --- checks
///
/// // fetch cells associated to each dart
/// let d1_cells = map.cells(d1);
/// let d2_cells = map.cells(d2);
/// let d3_cells = map.cells(d3);
///
/// // check dart-vertex association
/// assert_eq!(d1_cells.vertex_id, v1);
/// assert_eq!(d2_cells.vertex_id, v2);
/// assert_eq!(d3_cells.vertex_id, v3);
///
/// // check dart-face association
/// assert_eq!(d1_cells.face_id, fface_id);
/// assert_eq!(d2_cells.face_id, fface_id);
/// assert_eq!(d3_cells.face_id, fface_id);
///
/// // fetch all darts of the two-cell d2 belongs to
/// // i.e. the face
/// let two_cell = map.i_cell::<2>(d2);
///
/// // check topology of the face
/// // we make no assumption on the ordering of the result when using the i_cell method
/// assert!(two_cell.contains(&d1));
/// assert!(two_cell.contains(&d2));
/// assert!(two_cell.contains(&d3));
/// assert_eq!(two_cell.len(), 3);
///
/// // --- (a)
///
/// // add three new darts
/// let d4 = map.add_free_dart(); // 4
/// let d5 = map.add_free_dart(); // 5
/// let d6 = map.add_free_dart(); // 6
///
/// assert!(map.is_free(d4));
/// assert!(map.is_free(d5));
/// assert!(map.is_free(d6));
///
/// // create the corresponding three vertices
/// let v4 = map.add_vertex(Some([15.0, 0.0])); // v4
/// let v5 = map.add_vertices(2); // v5, v6
/// let v6 = v5 + 1;
/// map.set_vertex(v5, [5.0, 10.0]); // v5
/// map.set_vertex(v6, [15.0, 10.0]); // v6
/// // associate dart to vertices
/// map.set_vertexid(d4, v4);
/// map.set_vertexid(d5, v5);
/// map.set_vertexid(d6, v6);
/// // define beta values to form a second face
/// map.set_betas(d4, [d6, d5, NULL_DART_ID]); // beta 0 / 1 / 2 (d4) = d6 / d5 / null
/// map.set_betas(d5, [d4, d6, NULL_DART_ID]); // beta 0 / 1 / 2 (d5) = d4 / d6 / null
/// map.set_betas(d6, [d5, d4, NULL_DART_ID]); // beta 0 / 1 / 2 (d6) = d5 / d4 / null
///
/// let sface_id =  map.build_face(d6); // build the second face
///
/// // --- checks
///
/// // d4 & d2 are 2-free, hence can be 2-sewn together
/// assert!(map.is_i_free::<2>(d4));
/// assert!(map.is_i_free::<2>(d2));
///
/// // --- (b)
///
/// // 2-sew d2 & d4, stretching d4 to d2's spatial position
/// // this invalidates the face built before since vertex are overwritten
/// // if we used a StretchRight policy, the invalidated face would have been the first one
/// map.two_sew(d2, d4, SewPolicy::StretchLeft);
///
/// // --- checks
///
/// // check topological result
/// assert_eq!(map.beta::<2>(d2), d4);
/// assert_eq!(map.beta::<2>(d4), d2);
/// // check geometrical result
/// assert_eq!(map.vertexid(d2), map.vertexid(d5));
/// assert_eq!(map.vertexid(d3), map.vertexid(d4));
///
/// // --- (c)
///
/// // shift the position of d6 to build a square using the two faces
/// map.set_vertex(map.vertexid(d6), [10.0, 10.0]);
///
/// // --- (d)
///
/// // disconnect d2 & d4 for removal
/// map.one_unsew(d2, UnsewPolicy::Duplicate); // using unsew here allow proper vertices
/// map.one_unsew(d4, UnsewPolicy::Duplicate); // modifications
/// map.set_beta::<0>(d2, NULL_DART_ID);
/// map.set_beta::<0>(d4, NULL_DART_ID);
/// map.set_beta::<2>(d2, NULL_DART_ID);
/// map.set_beta::<2>(d4, NULL_DART_ID);
/// map.remove_free_dart(d2); // this checks if d2 is free for all i
/// map.remove_free_dart(d4); // this checks if d4 is free for all i
///
/// // reconnect d1/d5 & d6/d3 to form the new face
/// map.set_beta::<1>(d1, d5);
/// map.set_beta::<0>(d5, d1);
/// map.set_beta::<1>(d6, d3);
/// map.set_beta::<0>(d3, d6);
///
/// // rebuild the face
///
/// let new_face_id = map.build_face(d6);
///
/// // --- checks
///
/// // check associated face
/// assert_eq!(map.faceid(d1), new_face_id);
/// assert_eq!(map.faceid(d5), new_face_id);
/// assert_eq!(map.faceid(d6), new_face_id);
/// assert_eq!(map.faceid(d3), new_face_id);
///
/// // check dart positions
/// assert_eq!(*map.vertex(map.vertexid(d1)), [0.0, 0.0]);
/// assert_eq!(*map.vertex(map.vertexid(d5)), [0.0, 10.0]);
/// assert_eq!(*map.vertex(map.vertexid(d6)), [10.0, 10.0]);
/// assert_eq!(*map.vertex(map.vertexid(d3)), [10.0, 0.0]);
///
/// // check topology of the new face
/// let new_two_cell = map.i_cell::<2>(d3);
/// assert!(new_two_cell.contains(&d1));
/// assert!(new_two_cell.contains(&d5));
/// assert!(new_two_cell.contains(&d6));
/// assert!(new_two_cell.contains(&d3));
/// assert_eq!(new_two_cell.len(), 4);
///
/// ```
///
pub struct TwoMap<const N_MARKS: usize> {
    /// List of vertices making up the represented mesh
    vertices: Vec<Vertex2>,
    /// List of free vertex identifiers, i.e. empty spots
    /// in the current vertex list
    free_vertices: Vec<VertexIdentifier>,
    /// List of faces making up the represented mesh
    faces: Vec<Face>,
    /// Structure holding data related to darts (marks, associated cells)
    dart_data: DartData<N_MARKS>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list
    free_darts: Vec<DartIdentifier>,
    /// Array representation of the beta functions
    betas: Vec<[DartIdentifier; TWO_MAP_BETA]>,
    /// Current number of darts
    n_darts: usize,
    /// Current number of vertices
    n_vertices: usize,
}

macro_rules! stretch {
    ($slf: ident, $replaced: expr, $replacer: expr) => {
        $slf.dart_data.associated_cells[$replaced as usize].vertex_id =
            $slf.dart_data.associated_cells[$replacer as usize].vertex_id
    };
}

impl<const N_MARKS: usize> TwoMap<N_MARKS> {
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
    /// See [TwoMap] example.
    ///
    pub fn new(n_darts: usize, n_vertices: usize) -> Self {
        let vertices = vec![[0.0, 0.0]; n_vertices];
        let betas = vec![[0; TWO_MAP_BETA]; n_darts + 1];

        Self {
            vertices,
            free_vertices: Vec::with_capacity(n_vertices),
            faces: Vec::with_capacity(n_darts / 3),
            dart_data: DartData::new(n_darts),
            free_darts: Vec::with_capacity(n_darts + 1),
            betas,
            n_darts: n_darts + 1,
            n_vertices,
        }
    }

    // --- reading interfaces

    /// Return information about the current number of vertices.
    ///
    /// # Return / Panic
    ///
    /// Return a tuple of two elements:
    ///
    /// - the number of vertices
    /// - a boolean indicating whether there are free vertices or not
    ///
    /// The boolean essentially indicates if it is safe to access all
    /// vertex IDs in the `0..n_vertices` range.
    ///
    pub fn n_vertices(&self) -> (usize, bool) {
        (self.n_vertices, !self.free_vertices.is_empty())
    }

    /// Return information about the current number of darts.
    ///
    /// # Return / Panic
    ///
    /// Return a tuple of two elements:
    ///
    /// - the number of darts
    /// - a boolean indicating whether there are free darts or not
    ///
    /// The boolean essentially indicates if it is safe to access all
    /// dart IDs in the `0..n_darts` range.
    ///
    pub fn n_darts(&self) -> (usize, bool) {
        (self.n_darts, !self.free_darts.is_empty())
    }

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
    /// Return the identifier of the dart *d* such that *d = β<sub>i</sub>(dart)*. If
    /// the returned value is the null dart (i.e. a dart identifier equal to 0), this
    /// means that *dart* is i-free .
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn beta<const I: u8>(&self, dart_id: DartIdentifier) -> DartIdentifier {
        assert!(I < 3);
        self.betas[dart_id as usize][I as usize]
    }

    /// Fetch cells associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// # Return / Panic
    ///
    /// Return a reference to a [CellIdentifiers] structure that contain
    /// identifiers to the different **geometrical** i-cells *dart* models.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn cells(&self, dart_id: DartIdentifier) -> &CellIdentifiers {
        &self.dart_data.associated_cells[dart_id as usize]
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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn vertex(&self, vertex_id: VertexIdentifier) -> &Vertex2 {
        &self.vertices[vertex_id as usize]
    }

    /// Fetch face structure associated to a given identifier.
    ///
    /// # Arguments
    ///
    /// - `face_id: FaceIdentifier` -- Identifier of the given face.
    ///
    /// # Return / Panic
    ///
    /// Return a reference to the [Face] associated to the ID.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn face(&self, face_id: FaceIdentifier) -> &Face {
        &self.faces[face_id as usize]
    }

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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn vertexid(&self, dart_id: DartIdentifier) -> VertexIdentifier {
        self.dart_data.associated_cells[dart_id as usize].vertex_id
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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn faceid(&self, dart_id: DartIdentifier) -> FaceIdentifier {
        self.dart_data.associated_cells[dart_id as usize].face_id
    }

    /// Check if a given dart is i-free.
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
    /// Return a boolean indicating if *dart* is i-free, i.e.
    /// *β<sub>i</sub>(dart) = NullDart*.
    ///
    /// The function will panic if *I* is not 0, 1 or 2.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn is_free(&self, dart_id: DartIdentifier) -> bool {
        self.beta::<0>(dart_id) == NULL_DART_ID
            && self.beta::<1>(dart_id) == NULL_DART_ID
            && self.beta::<2>(dart_id) == NULL_DART_ID
    }

    // orbits / i-cells

    /// Return the identifiers of all dart composing an i-cell.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of *dart*.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Dimension of the cell of interest. *I* should
    /// be 0 (vertex), 1 (edge) or 2 (face) for a 2D map.
    ///
    /// # Return / Panic
    ///
    /// Returns a vector of IDs of the darts of the i-cell of *dart* (including
    /// *dart* at index 0).
    ///
    /// KNOWN ISSUE:
    ///
    /// - returning a vector is highly inefficient; a few alternatives to consider:
    /// ArrayVec or heapless Vec (requires a hard cap on the number of elements),
    /// an iterator...
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdentifier) -> Vec<DartIdentifier> {
        let mut cell: Vec<DartIdentifier> = vec![dart_id];
        let mut curr_dart = dart_id;
        match I {
            0 => {
                let mut completeness = true;
                // rotate around the vertex until we get back to the first dart
                while self.beta::<1>(self.beta::<2>(curr_dart)) != dart_id {
                    curr_dart = self.beta::<1>(self.beta::<2>(curr_dart));
                    cell.push(curr_dart);
                    if curr_dart == NULL_DART_ID {
                        completeness = false;
                        break; // stop if we land on the null dart
                    }
                }
                // if not complete, we need to rotate in the other direction to make sure
                // no dart is missing
                if !completeness {
                    curr_dart = self.beta::<2>(self.beta::<0>(dart_id));
                    while curr_dart != NULL_DART_ID {
                        cell.push(curr_dart);
                        curr_dart = self.beta::<2>(self.beta::<0>(curr_dart));
                    }
                }
            }
            1 => {
                // in the case of a 2-map, the 1-cell corresponds to [dart, beta_2(dart)]
                cell.push(self.beta::<2>(dart_id))
            }
            2 => {
                let mut completeness = true;
                // travel along the edges of the face until we get back to the first dart
                while self.beta::<1>(curr_dart) != dart_id {
                    curr_dart = self.beta::<1>(curr_dart);
                    cell.push(curr_dart);
                    if curr_dart == NULL_DART_ID {
                        completeness = false;
                        break; // stop if we land on the null dart
                    }
                }
                if !completeness {
                    curr_dart = self.beta::<0>(dart_id);
                    while curr_dart != NULL_DART_ID {
                        cell.push(curr_dart);
                        curr_dart = self.beta::<0>(curr_dart);
                    }
                }
            }
            _ => panic!(),
        }
        cell
    }

    // --- editing interfaces

    /// Add a new free dart to the combinatorial map.
    ///
    /// The dart is i-free for all i and is pushed to the list of existing
    /// darts, effectively making its identifier equal to the total number
    /// of darts (post-push).
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the created dart to allow for direct operations.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn add_free_dart(&mut self) -> DartIdentifier {
        let new_id = self.n_darts as DartIdentifier;
        self.n_darts += 1;
        self.dart_data.add_entry();
        self.betas.push([0; TWO_MAP_BETA]);
        new_id
    }

    /// Add multiple new free darts to the combinatorial map.
    ///
    /// All darts are i-free for all i and are pushed to the end of the list
    /// of existing darts.
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts to have.
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the first created dart to allow for direct operations.
    /// Darts are positionned on range `ID..ID+n_darts`.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn add_free_darts(&mut self, n_darts: usize) -> DartIdentifier {
        let new_id = self.n_darts as DartIdentifier;
        self.n_darts += n_darts;
        self.dart_data.add_entries(n_darts);
        self.betas.extend((0..n_darts).map(|_| [0; TWO_MAP_BETA]));
        new_id
    }

    /// Insert a new free dart to the combinatorial map.
    ///
    /// The dart is i-free for all i and may be inserted into a free spot in
    /// the existing dart list. If no free spots exist, it will be pushed to
    /// the end of the list.
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the created dart to allow for direct operations.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn insert_free_dart(&mut self) -> DartIdentifier {
        if let Some(new_id) = self.free_darts.pop() {
            self.dart_data.reset_entry(new_id);
            self.betas[new_id as usize] = [0; TWO_MAP_BETA];
            new_id
        } else {
            self.add_free_dart()
        }
    }

    /// Remove a free dart from the combinatorial map.
    ///
    /// The removed dart identifier is added to the list of free dart.
    /// This way of proceeding is necessary as the structure relies on
    /// darts indexing for encoding data, making reordering of any sort
    /// extremely costly.
    ///
    /// By keeping track of free spots in the dart arrays, we can prevent too
    /// much memory waste, although at the cost of locality of reference.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of the dart to remove.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn remove_free_dart(&mut self, dart_id: DartIdentifier) {
        assert!(self.is_free(dart_id));
        self.free_darts.push(dart_id);
        let b0d = self.beta::<0>(dart_id);
        let b1d = self.beta::<1>(dart_id);
        let b2d = self.beta::<2>(dart_id);
        self.betas[dart_id as usize] = [0; TWO_MAP_BETA];
        self.betas[b0d as usize][1] = 0 as DartIdentifier;
        self.betas[b1d as usize][0] = 0 as DartIdentifier;
        self.betas[b2d as usize][2] = 0 as DartIdentifier;
        // the following two lines are more safety than anything else
        // this prevents having to deal w/ artifacts in case of re-insertion
        self.dart_data.reset_entry(dart_id);
    }

    /// Add a vertex to the combinatorial map.
    ///
    /// The user can provide a [Vertex2] to use as the initial value of the
    /// added vertex.
    ///
    /// # Arguments
    ///
    /// - `vertex: Option<Vertex2>` -- Optional vertex value.
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the created vertex to allow for direct operations.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn add_vertex(&mut self, vertex: Option<Vertex2>) -> VertexIdentifier {
        let new_id = self.n_vertices as VertexIdentifier;
        self.n_vertices += 1;
        self.vertices.push(vertex.unwrap_or_default());
        new_id
    }

    /// Add multiple vertices to the combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `n_vertices: usize` -- Number of vertices to create.
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the first created vertex to allow for direct operations.
    /// Vertices are positionned on range `ID..ID+n_darts`.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn add_vertices(&mut self, n_vertices: usize) -> VertexIdentifier {
        let new_id = self.n_vertices as VertexIdentifier;
        self.n_vertices += n_vertices;
        self.vertices
            .extend((0..n_vertices).map(|_| Vertex2::default()));
        new_id
    }

    /// Insert a vertex in the combinatorial map.
    ///
    /// The vertex may be inserted into a free spot in the existing list. If no free
    /// spots exist, it will be pushed to the end of the list. The user can provide a
    /// [Vertex2] to use as the initial value of the added vertex.
    ///
    /// # Arguments
    ///
    /// - `vertex: Option<Vertex2>` -- Optional vertex value.
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the created dart to allow for direct operations.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn insert_vertex(&mut self, vertex: Option<Vertex2>) -> VertexIdentifier {
        if let Some(new_id) = self.free_vertices.pop() {
            self.set_vertex(new_id, vertex.unwrap_or_default()).unwrap();
            new_id
        } else {
            self.add_vertex(vertex)
        }
    }

    /// Remove a vertex from the combinatorial map.
    ///
    /// The removed vertex identifier is added to the list of free vertex.
    /// This way of proceeding is necessary as the structure relies on
    /// vertices indexing for encoding data, making reordering of any sort
    /// extremely costly.
    ///
    /// By keeping track of free spots in the vertices array, we can prevent too
    /// much memory waste, although at the cost of locality of reference.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to remove.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn remove_vertex(&mut self, vertex_id: VertexIdentifier) {
        self.free_vertices.push(vertex_id);
        // the following line is more safety than anything else
        // this prevents having to deal w/ artifacts in case of re-insertion
        // it also panics on OOB
        self.set_vertex(vertex_id, Vertex2::default()).unwrap();
    }

    /// Try to overwrite the given vertex with a new value.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to replace.
    /// - `vertex: Vertex2` -- New value for the vertex.
    ///
    /// # Return / Panic
    ///
    /// Return a result indicating if the vertex could be overwritten. The main reason
    /// of failure would be an out of bounds access.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_vertex(
        &mut self,
        vertex_id: VertexIdentifier,
        vertex: Vertex2,
    ) -> Result<(), MapError> {
        if let Some(val) = self.vertices.get_mut(vertex_id as usize) {
            *val = vertex;
            return Ok(());
        }
        Err(MapError::VertexOOB)
    }

    /// Set the values of the *β<sub>i</sub>* function of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `beta: DartIdentifier` -- Value of *β<sub>I</sub>(dart)*
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Dimension of the cell of interest. *I* should
    /// be 0 (vertex), 1 (edge) or 2 (face) for a 2D map.
    ///
    /// # Return / Panic
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_beta<const I: u8>(&mut self, dart_id: DartIdentifier, beta: DartIdentifier) {
        assert!(I < 3);
        self.betas[dart_id as usize][I as usize] = beta;
    }

    /// Set the values of the beta functions of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `betas: [DartIdentifier; 3]` -- Value of the images as
    ///   *[β<sub>0</sub>(dart), β<sub>1</sub>(dart), β<sub>2</sub>(dart)]*
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_betas(&mut self, dart_id: DartIdentifier, betas: [DartIdentifier; TWO_MAP_BETA]) {
        self.betas[dart_id as usize] = betas;
    }

    /// Set the vertex ID associated to a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `vertex_id: VertexIdentifier` -- Unique vertex identifier.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_vertexid(&mut self, dart_id: DartIdentifier, vertex_id: VertexIdentifier) {
        self.dart_data.associated_cells[dart_id as usize].vertex_id = vertex_id;
    }

    /// Set the face ID associated to a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `face_id: FaceIdentifier` -- Unique face identifier.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_faceid(&mut self, dart_id: DartIdentifier, face_id: FaceIdentifier) {
        self.dart_data.associated_cells[dart_id as usize].face_id = face_id;
    }

    /// 1-sewing operation.
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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn one_sew(
        &mut self,
        lhs_dart_id: DartIdentifier,
        rhs_dart_id: DartIdentifier,
        policy: SewPolicy,
    ) {
        // --- topological update

        // we could technically overwrite the value, but these assertions
        // makes it easier to assert algorithm correctness
        assert!(self.is_i_free::<1>(lhs_dart_id));
        assert!(self.is_i_free::<0>(rhs_dart_id));
        self.betas[lhs_dart_id as usize][1] = rhs_dart_id; // set beta_1(lhs_dart) to rhs_dart
        self.betas[rhs_dart_id as usize][0] = lhs_dart_id; // set beta_0(rhs_dart) to lhs_dart

        // --- geometrical update

        // in case of a 1-sew, we need to update the 0-cell geometry
        // of rhs_dart to ensure no vertex is duplicated

        // this operation only makes sense if lhs_dart is associated
        // to a fully defined edge, i.e. its image through beta2 is defined
        // & has a valid associated vertex (we assume the second condition
        // is valid if the first one is).
        let lid = self.beta::<2>(lhs_dart_id);
        if lid != NULL_DART_ID {
            match policy {
                SewPolicy::StretchLeft => {
                    stretch!(self, rhs_dart_id, lid);
                }
                SewPolicy::StretchRight => {
                    stretch!(self, lid, rhs_dart_id);
                }
                SewPolicy::StretchAverage => {
                    // this works under the assumption that a valid vertex is
                    // associated to rhs_dart
                    let lid_vertex = self.vertices[self.cells(lid).vertex_id as usize];
                    let rhs_vertex = self.vertices[self.cells(rhs_dart_id).vertex_id as usize];
                    self.vertices.push([
                        (lid_vertex[0] + rhs_vertex[0]) / 2.0,
                        (lid_vertex[1] + rhs_vertex[1]) / 2.0,
                    ]);
                    let new_id = (self.vertices.len() - 1) as VertexIdentifier;
                    stretch!(self, lid, new_id);
                    stretch!(self, rhs_dart_id, new_id);
                }
            }
        }
    }

    /// 2-sewing operation.
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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn two_sew(
        &mut self,
        lhs_dart_id: DartIdentifier,
        rhs_dart_id: DartIdentifier,
        policy: SewPolicy,
    ) {
        // --- topological update

        // we could technically overwrite the value, but these assertions
        // make it easier to assert algorithm correctness
        assert!(self.is_i_free::<2>(lhs_dart_id));
        assert!(self.is_i_free::<2>(rhs_dart_id));
        self.betas[lhs_dart_id as usize][2] = rhs_dart_id; // set beta_2(lhs_dart) to rhs_dart
        self.betas[rhs_dart_id as usize][2] = lhs_dart_id; // set beta_2(rhs_dart) to lhs_dart

        // --- geometrical update

        // in the case of a 2-sew, we need to ensure consistent orientation before completing
        // the operation
        // we can do this check while working on the embedded data we use those to verify
        // orientation

        // I swear this works
        // also, there is an urgent need for a custom vertex/vector type
        let l_is1free = self.is_i_free::<1>(lhs_dart_id);
        let r_is1free = self.is_i_free::<1>(rhs_dart_id);

        // depending on existing connections, different things are required
        match (l_is1free, r_is1free) {
            (true, true) => {} // do nothing
            (true, false) => {
                let b1rid = self.beta::<1>(rhs_dart_id);
                match policy {
                    SewPolicy::StretchLeft => {
                        stretch!(self, lhs_dart_id, b1rid)
                    }
                    SewPolicy::StretchRight => {
                        stretch!(self, b1rid, lhs_dart_id)
                    }
                    SewPolicy::StretchAverage => {
                        let vertex1 = self.vertices[self.cells(b1rid).vertex_id as usize];
                        let vertex2 = self.vertices[self.cells(lhs_dart_id).vertex_id as usize];

                        self.vertices.push([
                            (vertex1[0] + vertex2[0]) / 2.0,
                            (vertex1[1] + vertex2[1]) / 2.0,
                        ]);
                        let new_id = (self.vertices.len() - 1) as VertexIdentifier;

                        stretch!(self, b1rid, new_id);
                        stretch!(self, lhs_dart_id, new_id);
                    }
                }
            }
            (false, true) => {
                let b1lid = self.beta::<1>(lhs_dart_id);
                match policy {
                    SewPolicy::StretchLeft => {
                        stretch!(self, rhs_dart_id, b1lid)
                    }
                    SewPolicy::StretchRight => {
                        stretch!(self, b1lid, rhs_dart_id)
                    }
                    SewPolicy::StretchAverage => {
                        let vertex1 = self.vertices[self.cells(b1lid).vertex_id as usize];
                        let vertex2 = self.vertices[self.cells(rhs_dart_id).vertex_id as usize];

                        self.vertices.push([
                            (vertex1[0] + vertex2[0]) / 2.0,
                            (vertex1[1] + vertex2[1]) / 2.0,
                        ]);
                        let new_id = (self.vertices.len() - 1) as VertexIdentifier;

                        stretch!(self, b1lid, new_id);
                        stretch!(self, rhs_dart_id, new_id);
                    }
                }
            }
            (false, false) => {
                // ensure orientation consistency

                let b1lid = self.beta::<1>(lhs_dart_id);
                let b1rid = self.beta::<1>(rhs_dart_id);

                let b1_lvertex = self.vertices[self.cells(b1lid).vertex_id as usize];
                let lvertex = self.vertices[self.cells(lhs_dart_id).vertex_id as usize];
                let b1_rvertex = self.vertices[self.cells(b1rid).vertex_id as usize];
                let rvertex = self.vertices[self.cells(rhs_dart_id).vertex_id as usize];

                let lhs_vec = [b1_lvertex[0] - lvertex[0], b1_lvertex[1] - lvertex[1]];
                let rhs_vec = [b1_rvertex[0] - rvertex[0], b1_rvertex[1] - rvertex[1]];

                // dot product should be negative if the two darts have opposite direction
                let current = lhs_vec[0] * rhs_vec[0] + lhs_vec[1] * rhs_vec[1] < 0.0;

                if !current {
                    // we need reverse the orientation of the 2-cell
                    // i.e. swap values of beta 1 & beta 0
                    // for all elements connected to rhs & offset the
                    // associated vertices to keep consistency between
                    // placement & numbering
                    todo!("figure out how to reverse orientation of closed & open 2-cell")
                }

                match policy {
                    SewPolicy::StretchLeft => {
                        stretch!(self, rhs_dart_id, b1lid);
                        stretch!(self, b1rid, lhs_dart_id);
                    }
                    SewPolicy::StretchRight => {
                        stretch!(self, b1lid, rhs_dart_id);
                        stretch!(self, lhs_dart_id, b1rid);
                    }
                    SewPolicy::StretchAverage => {
                        let new_lvertex = [
                            (lvertex[0] + b1_rvertex[0]) / 2.0,
                            (lvertex[1] + b1_rvertex[1]) / 2.0,
                        ];
                        let new_rvertex = [
                            (rvertex[0] + b1_lvertex[0]) / 2.0,
                            (rvertex[1] + b1_lvertex[1]) / 2.0,
                        ];
                        self.vertices.push(new_lvertex);
                        self.vertices.push(new_rvertex);
                        let new_lid = self.vertices.len() - 2;
                        let new_rid = self.vertices.len() - 1;

                        stretch!(self, lhs_dart_id, new_lid);
                        stretch!(self, b1rid, new_lid);

                        stretch!(self, rhs_dart_id, new_rid);
                        stretch!(self, b1lid, new_rid);
                    }
                }
            }
        }
    }

    /// 1-unsewing operation.
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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn one_unsew(&mut self, lhs_dart_id: DartIdentifier, policy: UnsewPolicy) {
        // --- topological update

        // fetch id of beta_1(lhs_dart)
        let rhs_dart_id = self.beta::<1>(lhs_dart_id);
        self.betas[lhs_dart_id as usize][1] = 0; // set beta_1(lhs_dart) to NullDart
        self.betas[rhs_dart_id as usize][0] = 0; // set beta_0(rhs_dart) to NullDart

        // --- geometrical update
        match policy {
            UnsewPolicy::Duplicate => {
                // if the vertex was shared, duplicate it
                if self.i_cell::<0>(rhs_dart_id).len() > 1 {
                    let old_vertex = self.vertices[self.vertexid(rhs_dart_id) as usize];
                    self.vertices.push(old_vertex);
                    self.set_vertexid(rhs_dart_id, (self.vertices.len() - 1) as VertexIdentifier);
                }
            }
            UnsewPolicy::DoNothing => {}
        }
    }

    /// 2-unsewing operation.
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
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn two_unsew(&mut self, lhs_dart_id: DartIdentifier, policy: UnsewPolicy) {
        // --- topological update

        let rhs_dart_id = self.beta::<2>(lhs_dart_id);
        self.betas[lhs_dart_id as usize][2] = 0; // set beta_2(dart) to NullDart
        self.betas[rhs_dart_id as usize][2] = 0; // set beta_2(beta_2(dart)) to NullDart

        // --- geometrical update
        match policy {
            UnsewPolicy::Duplicate => {
                // if the vertex was shared, duplicate it
                // repeat on both ends of the edge
                let b1lid = self.beta::<1>(lhs_dart_id);
                if b1lid != NULL_DART_ID {
                    self.set_vertexid(rhs_dart_id, self.vertexid(b1lid));
                }
                let b1rid = self.beta::<1>(rhs_dart_id);
                if b1rid != NULL_DART_ID {
                    self.set_vertexid(lhs_dart_id, self.vertexid(b1rid));
                }
            }
            UnsewPolicy::DoNothing => {}
        }
    }

    /// Clear and rebuild the face list defined by the map.
    ///
    /// # Return / Panic
    ///
    /// Returns the number of faces built by the operation.
    ///
    /// # Example
    ///
    /// ```text
    ///
    /// ```
    ///
    pub fn build_all_faces(&mut self) -> usize {
        self.faces.clear();
        let mut n_faces = 0;
        // go through all darts ? update
        (0..self.n_darts as DartIdentifier).for_each(|id| {
            if !self.dart_data.was_marked(0, id) {
                let tmp = self.i_cell::<2>(id);
                if tmp.len() > 1 {
                    tmp.iter().for_each(|member| {
                        let _ = self.dart_data.was_marked(0, *member);
                    });
                    self.build_face(id);
                    n_faces += 1
                }
            }
        });
        n_faces
    }

    /// Build the geometrical face associated with a given dart
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of the dart
    ///
    /// # Return / Panic
    ///
    /// Return the ID of the created face to allow for direct operations.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn build_face(&mut self, dart_id: DartIdentifier) -> FaceIdentifier {
        let new_faceid = self.faces.len() as FaceIdentifier;
        self.set_faceid(dart_id, new_faceid);
        let mut part_one = vec![dart_id];
        let mut closed = true;
        let mut curr_dart = self.beta::<1>(dart_id);
        // search the face using beta1
        while curr_dart != dart_id {
            // if we encouter the null dart, it means the face is open
            if curr_dart == NULL_DART_ID {
                closed = false;
                break;
            }
            part_one.push(curr_dart);
            self.set_faceid(curr_dart, new_faceid);
            curr_dart = self.beta::<1>(curr_dart);
        }

        let res = if !closed {
            // if the face is open, we might have missed some darts
            // that were before the starting dart.
            curr_dart = self.beta::<0>(dart_id);
            let mut part_two = Vec::new();
            // search the face in the other direction using beta0
            while curr_dart != NULL_DART_ID {
                part_two.push(curr_dart);
                self.set_faceid(curr_dart, new_faceid);
                curr_dart = self.beta::<0>(curr_dart);
            }
            // to have the ordered face, we need to reverse the beta 0 part and
            // add the beta 1 part to its end
            part_two.reverse();
            part_two.extend(part_one);
            part_two
        } else {
            // if the face was closed
            // => we looped around its edges
            // => the list is already complete & ordered
            part_one
        };

        let face = Face {
            corners: res
                .iter()
                .map(|d_id| self.dart_data.associated_cells[*d_id as usize].vertex_id)
                .collect(),
            closed,
        };

        self.faces.push(face);
        new_faceid
    }
}

// --- 3-MAP

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
