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
/// - `faces: Vec<Face>` -- List of faces making up the represented mesh
/// - `dart_data: DartData<N_MARKS>` -- List of embedded data associated with darts
/// - `free_darts: Vec<DartIdentifier>` -- List of free darts identifiers, i.e. empty
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
/// - `const N_MARKS: usize` -- Number of marks used for search algorithms.
///   This corresponds to the number of search that can be done concurrently.
///
/// # Example
///
/// ```
/// use honeycomb_core::TwoMap;
///
/// // TODO: two examples to rule them all
///
/// ```
///
pub struct TwoMap<const N_MARKS: usize> {
    /// List of vertices making up the represented mesh
    vertices: Vec<Vertex2>,
    /// List of faces making up the represented mesh
    faces: Vec<Face>,
    /// Structure holding data related to darts (marks, associated cells)
    dart_data: DartData<N_MARKS>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list.
    free_darts: Vec<DartIdentifier>,
    /// Array representation of the beta functions
    ///
    /// This should eventually be replaced by a better
    /// structure, supported by benchmarking.
    betas: Vec<[DartIdentifier; TWO_MAP_BETA]>,
    /// Current number of darts
    n_darts: usize,
}

impl<const N_MARKS: usize> TwoMap<N_MARKS> {
    /// Creates a new 2D combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts composing the new combinatorial map.
    ///
    /// # Return / Panic
    ///
    /// Returns a combinatorial map containing:
    /// - `n_darts + 1`, the amount of darts wanted plus the null dart (at index 0).
    /// - 3 beta functions, *β<sub>0</sub>* being defined as the inverse of *β<sub>1</sub>*.
    /// - Default embed data associated to each dart.
    /// - An empty list of for vertices of the mesh.
    /// - An empty list of currently free darts. This may be used for dart creation.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn new(n_darts: usize) -> Self {
        let betas = vec![[0; TWO_MAP_BETA]; n_darts + 1];

        Self {
            dart_data: DartData::new(n_darts),
            betas,
            vertices: Vec::with_capacity(n_darts),
            faces: Vec::with_capacity(n_darts / 3),
            free_darts: Vec::with_capacity(n_darts + 1),
            n_darts: n_darts + 1,
        }
    }

    // --- reading interfaces

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
    /// Return a [CellIdentifiers] structure that contain identifiers to
    /// the different **geometrical** i-cells *dart* models.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn cells(&self, dart_id: DartIdentifier) -> CellIdentifiers {
        self.dart_data.associated_cells[dart_id as usize]
    }

    /// Fetch vertex associated to a given dart.
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
    pub fn vertex(&self, dart_id: DartIdentifier) -> VertexIdentifier {
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
    pub fn face(&self, dart_id: DartIdentifier) -> FaceIdentifier {
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
        macro_rules! stretch {
            ($replaced: expr, $replacer: expr) => {
                self.dart_data.associated_cells[$replaced as usize].vertex_id =
                    self.dart_data.associated_cells[$replacer as usize].vertex_id
            };
        }
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
                    stretch!(rhs_dart_id, lid);
                }
                SewPolicy::StretchRight => {
                    stretch!(lid, rhs_dart_id);
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
                    stretch!(lid, new_id);
                    stretch!(rhs_dart_id, new_id);
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
        macro_rules! stretch {
            ($replaced: expr, $replacer: expr) => {
                self.dart_data.associated_cells[$replaced as usize].vertex_id =
                    self.dart_data.associated_cells[$replacer as usize].vertex_id
            };
        }

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
                        stretch!(lhs_dart_id, b1rid)
                    }
                    SewPolicy::StretchRight => {
                        stretch!(b1rid, lhs_dart_id)
                    }
                    SewPolicy::StretchAverage => {
                        let vertex1 = self.vertices[self.cells(b1rid).vertex_id as usize];
                        let vertex2 = self.vertices[self.cells(lhs_dart_id).vertex_id as usize];

                        self.vertices.push([
                            (vertex1[0] + vertex2[0]) / 2.0,
                            (vertex1[1] + vertex2[1]) / 2.0,
                        ]);
                        let new_id = (self.vertices.len() - 1) as VertexIdentifier;

                        stretch!(b1rid, new_id);
                        stretch!(lhs_dart_id, new_id);
                    }
                }
            }
            (false, true) => {
                let b1lid = self.beta::<1>(lhs_dart_id);
                match policy {
                    SewPolicy::StretchLeft => {
                        stretch!(rhs_dart_id, b1lid)
                    }
                    SewPolicy::StretchRight => {
                        stretch!(b1lid, rhs_dart_id)
                    }
                    SewPolicy::StretchAverage => {
                        let vertex1 = self.vertices[self.cells(b1lid).vertex_id as usize];
                        let vertex2 = self.vertices[self.cells(rhs_dart_id).vertex_id as usize];

                        self.vertices.push([
                            (vertex1[0] + vertex2[0]) / 2.0,
                            (vertex1[1] + vertex2[1]) / 2.0,
                        ]);
                        let new_id = (self.vertices.len() - 1) as VertexIdentifier;

                        stretch!(b1lid, new_id);
                        stretch!(rhs_dart_id, new_id);
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
                        stretch!(rhs_dart_id, b1lid);
                        stretch!(b1rid, lhs_dart_id);
                    }
                    SewPolicy::StretchRight => {
                        stretch!(b1lid, rhs_dart_id);
                        stretch!(lhs_dart_id, b1rid);
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

                        stretch!(lhs_dart_id, new_lid);
                        stretch!(b1rid, new_lid);

                        stretch!(rhs_dart_id, new_rid);
                        stretch!(b1lid, new_rid);
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
    /// # Return / Panic
    ///
    /// The method may panic if *I* is neither 1 or 2.
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
                    let old_vertex = self.vertices[self.vertex(rhs_dart_id) as usize];
                    self.vertices.push(old_vertex);
                    self.set_vertex(rhs_dart_id, (self.vertices.len() - 1) as VertexIdentifier);
                }
            }
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
    /// # Return / Panic
    ///
    /// The method may panic if *I* is neither 1 or 2.
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
                    self.set_vertex(rhs_dart_id, self.vertex(b1lid));
                }
                let b1rid = self.beta::<1>(rhs_dart_id);
                if b1rid != NULL_DART_ID {
                    self.set_vertex(lhs_dart_id, self.vertex(b1rid));
                }
            }
        }
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
    pub fn set_vertex(&mut self, dart_id: DartIdentifier, vertex_id: VertexIdentifier) {
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
    pub fn set_face(&mut self, dart_id: DartIdentifier, face_id: FaceIdentifier) {
        self.dart_data.associated_cells[dart_id as usize].face_id = face_id;
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
    /// ```text
    ///
    /// ```
    ///
    pub fn build_face(&mut self, dart_id: DartIdentifier) -> FaceIdentifier {
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
        (self.faces.len() - 1) as FaceIdentifier
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
