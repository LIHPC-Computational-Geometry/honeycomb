//! Basic operations implementation
//!
//! This module contains code used to implement basic operations of combinatorial maps, such as
//! (but not limited to):
//!
//! - Dart addition / insertion / removal
//! - Beta function interfaces
//! - i-cell computations

// ------ IMPORTS

use super::CMAP2_BETA;
use crate::prelude::{
    CMap2, DartIdentifier, EdgeIdentifier, FaceIdentifier, Orbit2, OrbitPolicy, VertexIdentifier,
    NULL_DART_ID,
};
use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{EdgeCollection, FaceCollection, VertexCollection},
    geometry::CoordsFloat,
};

use std::collections::BTreeSet;

// ------ CONTENT

/// **Dart-related methods**
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
        self.attributes.extend_storages(1);
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
        self.attributes.extend_storages(n_darts);
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

/// **Beta-related methods**
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
}

/// **I-cell-related methods**
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
    ///   associated cell ID.
    /// - Cell IDs are not affected by the order of traversal of the map.
    /// - Because the ID is computed in real time, there is no need to store cell IDs and ensure
    ///   that the storage is consistent / up to date.
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
            .expect("E: unreachable") as VertexIdentifier
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
    ///   associated cell ID.
    /// - Cell IDs are not affected by the order of traversal of the map.
    /// - Because the ID is computed in real time, there is no need to store cell IDs and ensure
    ///   that the storage is consistent / up to date.
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
            .expect("E: unreachable") as EdgeIdentifier
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
    ///   associated cell ID.
    /// - Cell IDs are not affected by the order of traversal of the map.
    /// - Because the ID is computed in real time, there is no need to store cell IDs and ensure
    ///   that the storage is consistent / up to date.
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
            .expect("E: unreachable") as FaceIdentifier
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
    ///   2 (face) for a 2D map.
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
        let vids: BTreeSet<VertexIdentifier> = (1..self.n_darts as DartIdentifier)
            .filter_map(|d| {
                if self.unused_darts.contains(&d) {
                    None
                } else {
                    Some(self.vertex_id(d))
                }
            })
            .collect(); // duplicates are automatically handled when colelcting into a set
        VertexCollection::<'_, T>::new(self, vids)
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
        let eids: BTreeSet<EdgeIdentifier> = (1..self.n_darts as DartIdentifier)
            .filter_map(|d| {
                if self.unused_darts.contains(&d) {
                    None
                } else {
                    Some(self.edge_id(d))
                }
            })
            .collect(); // duplicates are automatically handled when colelcting into a set
        EdgeCollection::<'_, T>::new(self, eids)
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
        let fids: BTreeSet<EdgeIdentifier> = (1..self.n_darts as DartIdentifier)
            .filter_map(|d| {
                if self.unused_darts.contains(&d) {
                    None
                } else {
                    Some(self.face_id(d))
                }
            })
            .collect(); // duplicates are automatically handled when colelcting into a set
        FaceCollection::<'_, T>::new(self, fids)
    }
}
