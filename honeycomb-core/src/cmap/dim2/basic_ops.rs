//! Basic operations implementation
//!
//! This module contains code used to implement basic operations of combinatorial maps, such as
//! (but not limited to):
//!
//! - Dart addition / insertion / removal
//! - Beta function interfaces
//! - i-cell computations

// ------ IMPORTS

use crate::prelude::{
    CMap2, DartIdType, EdgeIdType, FaceIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID,
};
use crate::{attributes::UnknownAttributeStorage, geometry::CoordsFloat};
use itertools::Itertools;
use stm::{atomically, StmResult, Transaction};

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
        self.unused_darts.iter().filter(|v| v.read_atomic()).count()
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
    pub fn add_free_dart(&mut self) -> DartIdType {
        let new_id = self.n_darts as DartIdType;
        self.n_darts += 1;
        self.betas.extend(1);
        self.unused_darts.extend(1);
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
    pub fn add_free_darts(&mut self, n_darts: usize) -> DartIdType {
        let new_id = self.n_darts as DartIdType;
        self.n_darts += n_darts;
        self.betas.extend(n_darts);
        self.unused_darts.extend(n_darts);
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
    pub fn insert_free_dart(&mut self) -> DartIdType {
        if let Some((new_id, _)) = self
            .unused_darts
            .iter()
            .enumerate()
            .find(|(_, u)| u.read_atomic())
        {
            atomically(|trans| self.unused_darts[new_id as DartIdType].write(trans, false));
            new_id as DartIdType
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
    pub fn remove_free_dart(&mut self, dart_id: DartIdType) {
        atomically(|trans| {
            assert!(self.is_free(dart_id)); // all beta images are 0
            assert!(!self.unused_darts[dart_id as DartIdType].replace(trans, true)?);
            Ok(())
        });
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
    pub fn beta<const I: u8>(&self, dart_id: DartIdType) -> DartIdType {
        assert!(I < 3);
        self.betas[(I, dart_id)].read_atomic()
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
    pub fn beta_rt(&self, i: u8, dart_id: DartIdType) -> DartIdType {
        assert!(i < 3);
        match i {
            0 => self.beta::<0>(dart_id),
            1 => self.beta::<1>(dart_id),
            2 => self.beta::<2>(dart_id),
            _ => unreachable!(),
        }
    }

    /// Compute the value of the i-th beta function of a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    /// - `const I: u8` -- Index of the beta function. *I* should be 0, 1 or 2 for a 2D map.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    ///
    /// # Panics
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn beta_transac<const I: u8>(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmResult<DartIdType> {
        assert!(I < 3);
        self.betas[(I, dart_id)].read(trans)
    }

    /// Compute the value of the i-th beta function of a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of a given dart.
    /// - `i: u8` -- Index of the beta function. *i* should be 0, 1 or 2 for a 2D map.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    ///
    /// # Panics
    ///
    /// The method will panic if *i* is not 0, 1 or 2.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn beta_rt_transac(
        &self,
        trans: &mut Transaction,
        i: u8,
        dart_id: DartIdType,
    ) -> StmResult<DartIdType> {
        assert!(i < 3);
        match i {
            0 => self.beta_transac::<0>(trans, dart_id),
            1 => self.beta_transac::<1>(trans, dart_id),
            2 => self.beta_transac::<2>(trans, dart_id),
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
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdType) -> bool {
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
    pub fn is_free(&self, dart_id: DartIdType) -> bool {
        self.beta::<0>(dart_id) == NULL_DART_ID
            && self.beta::<1>(dart_id) == NULL_DART_ID
            && self.beta::<2>(dart_id) == NULL_DART_ID
    }
}

/// **I-cell-related methods**
impl<T: CoordsFloat> CMap2<T> {
    /// Compute the associated vertex ID of a given dart.
    ///
    /// This corresponds to the minimum dart ID among darts composing the topological vertex.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn vertex_id(&self, dart_id: DartIdType) -> VertexIdType {
        atomically(|trans| self.vertex_id_transac(trans, dart_id))
    }

    /// Compute the associated vertex ID of a given dart.
    ///
    /// This corresponds to the minimum dart ID among darts composing the topological vertex.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    pub fn vertex_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmResult<VertexIdType> {
        // min encountered / current dart
        let mut min = dart_id;
        let mut crt = self.betas[(1, self.betas[(2, dart_id)].read(trans)?)].read(trans)?;

        // we first iterate in direct direction (B1oB2)
        while crt != NULL_DART_ID && crt != dart_id {
            min = min.min(crt);
            crt = self.betas[(1, self.betas[(2, crt)].read(trans)?)].read(trans)?;
        }
        // if we landed on the null dart, the vertex is open
        // we need to iterate in the opposite dir (B2oB0)
        if crt == NULL_DART_ID {
            crt = self.betas[(2, self.betas[(0, dart_id)].read(trans)?)].read(trans)?;
            while crt != NULL_DART_ID {
                min = min.min(crt);
                crt = self.betas[(2, self.betas[(0, crt)].read(trans)?)].read(trans)?;
            }
        }

        Ok(min)
    }

    /// Compute the associated edge ID of a given dart.
    ///
    /// This corresponds to the minimum dart ID among darts composing the topological edge.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn edge_id(&self, dart_id: DartIdType) -> EdgeIdType {
        atomically(|trans| self.edge_id_transac(trans, dart_id))
    }

    /// Compute the associated edge ID of a given dart.
    ///
    /// This corresponds to the minimum dart ID among darts composing the topological edge.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    pub fn edge_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmResult<EdgeIdType> {
        // optimizing this one bc I'm tired
        let b2 = self.beta_transac::<2>(trans, dart_id)?;
        if b2 == NULL_DART_ID {
            Ok(dart_id as EdgeIdType)
        } else {
            Ok(b2.min(dart_id) as EdgeIdType)
        }
    }

    /// Compute the associated face ID of a given dart.
    ///
    /// This corresponds to the minimum dart ID among darts composing the topological face.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn face_id(&self, dart_id: DartIdType) -> FaceIdType {
        atomically(|trans| self.face_id_transac(trans, dart_id))
    }

    /// Compute the associated face ID of a given dart.
    ///
    /// This corresponds to the minimum dart ID among darts composing the topological face.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    pub fn face_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmResult<FaceIdType> {
        // min encountered / current dart
        let mut min = dart_id;
        let mut crt = self.beta_transac::<1>(trans, dart_id)?;

        // we first iterate in direct direction (B1)
        while crt != NULL_DART_ID && crt != dart_id {
            min = min.min(crt);
            crt = self.beta_transac::<1>(trans, crt)?;
        }
        // if we landed on the null dart, the face is open
        // we need to iterate in the opposite dir (B0)
        if crt == NULL_DART_ID {
            crt = self.beta_transac::<0>(trans, dart_id)?;
            while crt != NULL_DART_ID {
                min = min.min(crt);
                crt = self.beta_transac::<0>(trans, crt)?;
            }
        }

        Ok(min)
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
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdType) -> Orbit2<T> {
        assert!(I < 3);
        match I {
            0 => Orbit2::<'_, T>::new(self, OrbitPolicy::Vertex, dart_id),
            1 => Orbit2::<'_, T>::new(self, OrbitPolicy::Edge, dart_id),
            2 => Orbit2::<'_, T>::new(self, OrbitPolicy::Face, dart_id),
            _ => unreachable!(),
        }
    }

    /// Return an iterator over IDs of all the map's faces.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn iter_vertices(&self) -> impl Iterator<Item = VertexIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(|(d, unused)| {
                if unused.read_atomic() {
                    None
                } else {
                    Some(self.vertex_id(d))
                }
            })
            .unique()
    }

    /// Return an iterator over IDs of all the map's edges.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn iter_edges(&self) -> impl Iterator<Item = EdgeIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(|(d, unused)| {
                if unused.read_atomic() {
                    None
                } else {
                    Some(self.edge_id(d))
                }
            })
            .unique()
    }

    /// Return an iterator over IDs of all the map's faces.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn iter_faces(&self) -> impl Iterator<Item = FaceIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(|(d, unused)| {
                if unused.read_atomic() {
                    None
                } else {
                    Some(self.face_id(d))
                }
            })
            .unique()
    }
}
