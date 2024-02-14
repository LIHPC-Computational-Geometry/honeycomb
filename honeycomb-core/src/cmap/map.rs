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

use crate::{DartIdentifier, FaceIdentifier, VertexIdentifier};

use super::{
    dart::{CellIdentifiers, DartData},
    embed::{SewPolicy, UnsewPolicy, Vertex},
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
/// - `vertices: Vec<Vertex>`: List of vertices making up the represented mesh
/// - `cells: Vec<DartCells>`: List of associated cells of each dart
/// - `darts: Vec<Dart>`: List of darts composing the map
/// - `free_darts: Vec<DartIdentifier>`: List of free darts identifiers, i.e. empty
///   spots in the current dart list.
/// - `betas: Vec<[DartIdentifier; 3]>`: Array representation of the beta functions
///
/// Note that we encode *β<sub>0</sub>* as the inverse function of *β<sub>1</sub>*.
/// This is extremely useful (read *required*) to implement correct and efficient
/// i-cell computation. Additionally, while *β<sub>0</sub>* can be accessed using
/// the [Self::beta] method, we do not define 0-sew or 0-unsew operations.
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
    vertices: Vec<Vertex>,
    /// Structure holding data related to darts (marks, associated cells)
    dart_data: DartData<N_MARKS>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list.
    free_darts: Vec<DartIdentifier>,
    /// Array representation of the beta functions.
    ///
    /// This should eventually be replaced by a better
    /// structure, supported by benchmarking.
    betas: Vec<[DartIdentifier; TWO_MAP_BETA]>,
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
            free_darts: Vec::with_capacity(n_darts + 1),
        }
    }

    // --- reading interfaces

    /// Compute the value of the I-th beta function of a given dart.
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
    /// Return the identifier of the dart *d* such that *d = β<sub>I</sub>(dart)*. If
    /// the returned value is the null dart, this means that *dart* is I-free.
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
    /// Return a [DartCells] structure that contain identifiers to
    /// the different i-cells *dart* models.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn cell_of(&self, dart_id: DartIdentifier) -> CellIdentifiers {
        self.dart_data.associated_cells[dart_id as usize]
    }

    /// Check if a given dart is I-free.
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
    /// Return a boolean indicating if *dart* is I-free, i.e.
    /// *β<sub>I</sub>(dart) = NullDart*.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdentifier) -> bool {
        self.beta::<I>(dart_id) == 0
    }

    /// Check if a given dart is I-free, for all I.
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
        self.beta::<0>(dart_id) == 0 && self.beta::<1>(dart_id) == 0 && self.beta::<2>(dart_id) == 0
    }

    // orbits / i-cells

    /// Description
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
    /// Returns a vector of IDs of the darts of the I-cell of *dart* (including
    /// *dart* at index 0).
    ///
    /// KNOWN ISSUE:
    ///
    /// - if beta I is a partial permutation, the returned cell might not be
    /// complete, this is especially a problem for beta 1 since 0-cell and
    /// 2-cell might be incomplete.
    /// - returning a vector is highly inefficient; a few alternatives to consider:
    /// ArrayVec or heapless Vec (requires a hard cap on the number of elements),
    /// an iterator...
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn i_cell_of<const I: usize>(&self, dart_id: DartIdentifier) -> Vec<DartIdentifier> {
        let mut cell: Vec<DartIdentifier> = vec![dart_id];
        let mut curr_dart = dart_id;
        match I {
            0 => {
                // rotate around the vertex until we get back to the first dart
                while self.beta::<1>(self.beta::<2>(curr_dart)) != dart_id {
                    curr_dart = self.beta::<1>(self.beta::<2>(curr_dart));
                    cell.push(curr_dart);
                    if curr_dart == 0 {
                        break; // stop if we land on the null dart
                    }
                }
            }
            1 => {
                // in the case of a 2-map, the 1-cell corresponds to [dart, beta_2(dart)]
                cell.push(self.beta::<2>(dart_id))
            }
            2 => {
                // travel along the edges of the face until we get back to the first dart
                while self.beta::<1>(curr_dart) != dart_id {
                    curr_dart = self.beta::<1>(curr_dart);
                    cell.push(curr_dart);
                    if curr_dart == 0 {
                        break; // stop if we land on the null dart
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
        let new_id = self.dart_data.associated_cells.len() as DartIdentifier;
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
        self.betas[dart_id as usize] = [0; TWO_MAP_BETA];
        // the following two lines are more safety than anything else
        // this prevents having to deal w/ artifacts in case of re-insertion
        self.dart_data.reset_entry(dart_id);
    }

    /// i-sewing operation.
    ///
    /// This operation corresponds to *coherently linking* two darts via
    /// the *β<sub>i</sub>* function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// After the sewing operation, these darts will verify
    /// `*β<sub>i</sub>*(lhs_dart) == rhs_dart`.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should
    /// be either 1 or 2 in the case of a 2D map. Note that *β<sub>0</sub>*
    /// will be updated in case of a 1-sew.
    ///
    /// # Return / Panic
    ///
    /// The method may panic if *I* is neither 1 or 2.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn i_sew<const I: u8>(
        &mut self,
        lhs_dart_id: DartIdentifier,
        rhs_dart_id: DartIdentifier,
        policy: SewPolicy,
    ) {
        match I {
            1 => {
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
                let zero_cell = self.i_cell_of::<0>(rhs_dart_id);
                if zero_cell.len() > 1 {
                    // if there was an existing 0-cell, update according to sewing policy
                    match policy {
                        // "move" rhs_dart to existing 0-cell
                        SewPolicy::MergeToExisting => {
                            self.dart_data.associated_cells[rhs_dart_id as usize].vertex_id =
                                self.cell_of(zero_cell[1]).vertex_id
                        }
                    };
                }
            }
            2 => {
                // --- topological update

                // we could technically overwrite the value, but these assertions
                // make it easier to assert algorithm correctness
                assert!(self.is_i_free::<2>(lhs_dart_id));
                assert!(self.is_i_free::<2>(rhs_dart_id));
                self.betas[lhs_dart_id as usize][1] = rhs_dart_id; // set beta_2(lhs_dart) to rhs_dart
                self.betas[rhs_dart_id as usize][1] = lhs_dart_id; // set beta_2(rhs_dart) to lhs_dart

                // --- geometrical update
            }
            _ => panic!(),
        }
    }

    /// i-unsewing operation.
    ///
    /// This operation corresponds to *coherently separating* two darts linked
    /// via the *β<sub>i</sub>* function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to separate.
    ///
    /// Note that we do not need to take two darts as arguments since the
    /// second dart can be obtained through the *β<sub>i</sub>* function.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should
    /// be either 1 or 2 in the case of a 2D map. Note that *β<sub>0</sub>*
    /// will be updated in case of a 1-unsew.
    ///
    /// # Return / Panic
    ///
    /// The method may panic if *I* is neither 1 or 2.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn i_unsew<const I: u8>(&mut self, lhs_dart_id: DartIdentifier, policy: UnsewPolicy) {
        match I {
            1 => {
                // --- topological update

                // fetch id of beta_1(lhs_dart)
                let rhs_dart_id = self.beta::<1>(lhs_dart_id);
                self.betas[lhs_dart_id as usize][1] = 0; // set beta_1(lhs_dart) to NullDart
                self.betas[rhs_dart_id as usize][0] = 0; // set beta_0(rhs_dart) to NullDart

                // --- geometrical update
                match policy {
                    UnsewPolicy::Duplicate => {
                        let old_vertex = self.vertices[self.cell_of(rhs_dart_id).vertex_id];
                        self.vertices.push(old_vertex);
                        self.set_d_vertex(rhs_dart_id, self.vertices.len() - 1);
                    }
                }
            }
            2 => {
                // --- topological update

                let opp = self.beta::<2>(lhs_dart_id);
                self.betas[lhs_dart_id as usize][2] = 0; // set beta_2(dart) to NullDart
                self.betas[opp as usize][2] = 0; // set beta_2(beta_2(dart)) to NullDart

                // --- geometrical update
            }
            _ => panic!(),
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
    pub fn set_d_betas(&mut self, dart_id: DartIdentifier, betas: [DartIdentifier; TWO_MAP_BETA]) {
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
    pub fn set_d_vertex(&mut self, dart_id: DartIdentifier, vertex_id: VertexIdentifier) {
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
    pub fn set_d_face(&mut self, dart_id: DartIdentifier, face_id: FaceIdentifier) {
        self.dart_data.associated_cells[dart_id as usize].face_id = face_id;
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
