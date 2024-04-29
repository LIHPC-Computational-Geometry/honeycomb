//! Map objects
//!
//! This module contains code for the two main structures provided
//! by the crate:
//!
//! - [`CMap2`], a 2D combinatorial map implementation
//!
//! The definitions are re-exported, direct interaction with this module
//! should be minimal, if existing at all.

// ------ MODULE DECLARATIONS

mod link_and_sew;
mod structure;
#[cfg(any(doc, feature = "utils"))]
mod utils;

// ------ RE-EXPORTS

pub use structure::CMap2;

// ------ IMPORTS

use crate::{
    CoordsFloat, DartIdentifier, EdgeCollection, EdgeIdentifier, FaceCollection, FaceIdentifier,
    Orbit2, OrbitPolicy, Vertex2, VertexCollection, VertexIdentifier, NULL_DART_ID,
};
use std::collections::BTreeSet;

// ------ CONTENT

/// Map-level error enum
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

// --- dart-related code
impl<T: CoordsFloat> CMap2<T> {
    // --- read

    /// Return information about the current number of darts.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_darts(&self) -> usize {
        self.n_darts
    }

    /// Return information about the current number of unused darts.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_unused_darts(&self) -> usize {
        self.unused_darts.len()
    }

    // --- edit

    /// Add a new free dart to the combinatorial map.
    ///
    /// The dart is i-free for all i and is pushed to the list of existing darts, effectively
    /// making its identifier equal to the total number of darts (post-push).
    ///
    /// # Return
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
    /// # Return
    ///
    /// Return the `ID` of the first created dart to allow for direct operations. Darts are
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
    /// # Return
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
    /// # Panics
    ///
    /// This method may panic if:
    ///
    /// - The dart is not *i*-free for all *i*.
    /// - The dart is already marked as unused (Refer to [`Self::remove_vertex`] documentation for
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
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should be 0, 1 or 2 for a 2D map.
    ///
    /// # Return
    ///
    /// Return the identifier of the dart *d* such that *d = β<sub>i</sub>(dart)*. If the returned
    /// value is the null dart (i.e. a dart ID equal to 0), this means that the dart is i-free.
    ///
    /// # Panics
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn beta<const I: u8>(&self, dart_id: DartIdentifier) -> DartIdentifier {
        assert!(I < 3);
        self.betas[dart_id as usize][I as usize]
    }

    /// Compute the value of the i-th beta function of a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    /// - `i: u8` -- Index of the beta function. *i* should be 0, 1 or 2 for a 2D map.
    ///
    /// # Return
    ///
    /// Return the identifier of the dart *d* such that *d = β<sub>i</sub>(dart)*. If the returned
    /// value is the null dart (i.e. a dart ID equal to 0), this means that the dart is i-free.
    ///
    /// # Panics
    ///
    /// The method will panic if *i* is not 0, 1 or 2.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
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
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should be 0, 1 or 2 for a 2D map.
    ///
    /// # Return
    ///
    /// Return a boolean indicating if the dart is i-free, i.e.
    /// *β<sub>i</sub>(dart) = `NULL_DART_ID`*.
    ///
    /// # Panics
    ///
    /// The function will panic if *I* is not 0, 1 or 2.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdentifier) -> bool {
        self.beta::<I>(dart_id) == NULL_DART_ID
    }

    /// Check if a given dart is i-free, for all i.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    ///
    /// # Return
    ///
    /// Return a boolean indicating if the dart is 0-free, 1-free **and** 2-free.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
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
    #[allow(clippy::missing_panics_doc)]
    /// Fetch vertex identifier associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    ///
    /// # Return
    ///
    /// Return the identifier of the associated vertex.
    ///
    /// ## Note on cell identifiers
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
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn vertex_id(&self, dart_id: DartIdentifier) -> VertexIdentifier {
        // unwraping the result is safe because the orbit is always non empty
        Orbit2::<'_, T>::new(self, OrbitPolicy::Vertex, dart_id)
            .min()
            .unwrap() as VertexIdentifier
    }

    #[allow(clippy::missing_panics_doc)]
    /// Fetch edge associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    ///
    /// # Return
    ///
    /// Return the identifier of the associated edge.
    ///
    /// ## Note on cell identifiers
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
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn edge_id(&self, dart_id: DartIdentifier) -> EdgeIdentifier {
        // unwraping the result is safe because the orbit is always non empty
        Orbit2::<'_, T>::new(self, OrbitPolicy::Edge, dart_id)
            .min()
            .unwrap() as EdgeIdentifier
    }

    #[allow(clippy::missing_panics_doc)]
    /// Fetch face associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    ///
    /// # Return
    ///
    /// Return the identifier of the associated face.
    ///
    /// ## Note on cell identifiers
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
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn face_id(&self, dart_id: DartIdentifier) -> FaceIdentifier {
        // unwraping the result is safe because the orbit is always non empty
        Orbit2::<'_, T>::new(self, OrbitPolicy::Face, dart_id)
            .min()
            .unwrap() as FaceIdentifier
    }

    /// Return an [`Orbit2`] object that can be used to iterate over darts of an i-cell.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Dimension of the cell of interest. *I* should be 0 (vertex), 1 (edge) or
    /// 2 (face) for a 2D map.
    ///
    /// # Return
    ///
    /// Returns an [`Orbit2`] that can be iterated upon to retrieve all dart member of the cell. Note
    /// that **the dart passed as an argument is included as the first element of the returned
    /// orbit**.
    ///
    /// # Panics
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdentifier) -> Orbit2<T> {
        assert!(I < 3);
        match I {
            0 => Orbit2::<'_, T>::new(self, OrbitPolicy::Vertex, dart_id),
            1 => Orbit2::<'_, T>::new(self, OrbitPolicy::Edge, dart_id),
            2 => Orbit2::<'_, T>::new(self, OrbitPolicy::Face, dart_id),
            _ => unreachable!(),
        }
    }

    /// Return a collection of all the map's vertices.
    ///
    /// # Return
    ///
    /// Return a [`VertexCollection`] object containing a list of vertex identifiers, whose validity
    /// is ensured through an implicit lifetime condition on the structure and original map.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
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
                    Orbit2::<'_, T>::new(self, OrbitPolicy::Vertex, dart_id).for_each(|did| {
                        marked.insert(did);
                    });
                }
            });
        VertexCollection::<'_, T>::new(self, vertex_ids)
    }

    /// Return a collection of all the map's edges.
    ///
    /// # Return
    ///
    /// Return an [`EdgeCollection`] object containing a list of edge identifiers, whose validity
    /// is ensured through an implicit lifetime condition on the structure and original map.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
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
                    Orbit2::<'_, T>::new(self, OrbitPolicy::Edge, dart_id).for_each(|did| {
                        marked.insert(did);
                    });
                }
            });
        EdgeCollection::<'_, T>::new(self, edge_ids)
    }

    /// Return a collection of all the map's faces.
    ///
    /// # Return
    ///
    /// Return a [`FaceCollection`] object containing a list of face identifiers, whose validity
    /// is ensured through an implicit lifetime condition on the structure and original map.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
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
                    Orbit2::<'_, T>::new(self, OrbitPolicy::Face, dart_id).for_each(|did| {
                        marked.insert(did);
                    });
                }
            });
        FaceCollection::<'_, T>::new(self, face_ids)
    }
}

// --- vertex attributes
// this should eventually be replaced by a generalized structure to handle
// different kind of attributes for all the i-cells.
impl<T: CoordsFloat> CMap2<T> {
    /// Return the current number of vertices.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_vertices(&self) -> usize {
        self.vertices.n_attributes()
    }

    /// Fetch vertex value associated to a given identifier.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the given vertex.
    ///
    /// # Return
    ///
    /// Return a reference to the [Vertex2] associated to the ID.
    ///
    /// # Panics
    ///
    /// The method may panic if no vertex is associated to the specified index, or the ID lands
    /// out of bounds.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn vertex(&self, vertex_id: VertexIdentifier) -> Vertex2<T> {
        self.vertices.get(&vertex_id).unwrap()
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
        self.vertices.insert(&vertex_id, vertex.into());
    }

    #[allow(clippy::missing_errors_doc)]
    /// Remove a vertex from the combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to remove.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(v: Vertex2)` -- The vertex was successfully removed & its value was returned
    /// - `Err(CMapError::UndefinedVertexID)` -- The vertex was not found in the internal storage
    ///
    pub fn remove_vertex(&mut self, vertex_id: VertexIdentifier) -> Result<Vertex2<T>, CMapError> {
        if let Some(val) = self.vertices.remove(&vertex_id) {
            return Ok(val);
        }
        Err(CMapError::UndefinedVertex)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Try to overwrite the given vertex with a new value.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to replace.
    /// - `vertex: impl<Into<Vertex2>>` -- New value for the vertex.
    ///
    /// # Return / Errors
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
        if let Some(val) = self.vertices.replace(&vertex_id, vertex.into()) {
            return Ok(val);
        };
        Err(CMapError::UndefinedVertex)
    }
}

// ------ TESTS
#[cfg(test)]
mod tests;
