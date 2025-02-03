//! Basic operations implementation
//!
//! This module contains code used to implement basic operations of combinatorial maps, such as
//! (but not limited to):
//!
//! - Dart addition / insertion / removal
//! - Beta function interfaces
//! - i-cell computations

// ------ IMPORTS

use std::collections::{HashSet, VecDeque};

use crate::prelude::{
    CMap2, DartIdType, EdgeIdType, FaceIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID,
};
use crate::stm::{atomically, StmClosureResult, Transaction};
use crate::{attributes::UnknownAttributeStorage, geometry::CoordsFloat};

// ------ CONTENT

/// **Dart-related methods**
impl<T: CoordsFloat> CMap2<T> {
    // --- read

    /// Return the current number of darts.
    #[must_use = "unused return value"]
    pub fn n_darts(&self) -> usize {
        self.n_darts
    }

    /// Return the current number of unused darts.
    #[must_use = "unused return value"]
    pub fn n_unused_darts(&self) -> usize {
        self.unused_darts.iter().filter(|v| v.read_atomic()).count()
    }

    // --- edit

    /// Add a new free dart to the map.
    ///
    /// # Return
    ///
    /// Return the ID of the new dart.
    pub fn add_free_dart(&mut self) -> DartIdType {
        let new_id = self.n_darts as DartIdType;
        self.n_darts += 1;
        self.betas.extend(1);
        self.unused_darts.extend(1);
        self.vertices.extend(1);
        self.attributes.extend_storages(1);
        new_id
    }

    /// Add `n_darts` new free darts to the map.
    ///
    /// # Return
    ///
    /// Return the ID of the first new dart. Other IDs are in the range `ID..ID+n_darts`.
    pub fn add_free_darts(&mut self, n_darts: usize) -> DartIdType {
        let new_id = self.n_darts as DartIdType;
        self.n_darts += n_darts;
        self.betas.extend(n_darts);
        self.unused_darts.extend(n_darts);
        self.vertices.extend(n_darts);
        self.attributes.extend_storages(n_darts);
        new_id
    }

    /// Insert a new free dart in the map.
    ///
    /// The dart may be inserted into an unused spot of the existing dart list. If no free spots
    /// exist, it will be pushed to the end of the list.
    ///
    /// # Return
    ///
    /// Return the ID of the new dart.
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

    /// Remove a free dart from the map.
    ///
    /// The removed dart identifier is added to the list of free dart. This way of proceeding is
    /// necessary as the structure relies on darts indexing for encoding data, making reordering of
    /// any sort extremely costly.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- Identifier of the dart to remove.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - the dart is not *i*-free for all *i*,
    /// - the dart is already marked as unused.
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

    /// Return  β<sub>`I`</sub>(`dart_id`).
    ///
    /// # Panics
    ///
    /// The method will panic if `I` is not 0, 1 or 2.
    #[must_use = "unused return value"]
    pub fn beta<const I: u8>(&self, dart_id: DartIdType) -> DartIdType {
        assert!(I < 3);
        self.betas[(I, dart_id)].read_atomic()
    }

    /// Return  β<sub>`i`</sub>(`dart_id`).
    ///
    /// # Panics
    ///
    /// The method will panic if `i` is not 0, 1 or 2.
    #[must_use = "unused return value"]
    pub fn beta_rt(&self, i: u8, dart_id: DartIdType) -> DartIdType {
        assert!(i < 3);
        match i {
            0 => self.beta::<0>(dart_id),
            1 => self.beta::<1>(dart_id),
            2 => self.beta::<2>(dart_id),
            _ => unreachable!(),
        }
    }

    /// Return  β<sub>`I`</sub>(`dart_id`).
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method will panic if `I` is not 0, 1 or 2.
    pub fn beta_transac<const I: u8>(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<DartIdType> {
        assert!(I < 3);
        self.betas[(I, dart_id)].read(trans)
    }

    /// Return  β<sub>`i`</sub>(`dart_id`).
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method will panic if `i` is not 0, 1 or 2.
    pub fn beta_rt_transac(
        &self,
        trans: &mut Transaction,
        i: u8,
        dart_id: DartIdType,
    ) -> StmClosureResult<DartIdType> {
        assert!(i < 3);
        match i {
            0 => self.beta_transac::<0>(trans, dart_id),
            1 => self.beta_transac::<1>(trans, dart_id),
            2 => self.beta_transac::<2>(trans, dart_id),
            _ => unreachable!(),
        }
    }

    /// Check if a given dart is `I`-free.
    ///
    /// # Return
    ///
    /// Return a boolean indicating if the dart is `I`-free, i.e.:
    /// - `true` if β<sub>`I`</sub>(`dart_id`) = `NULL_DART_ID`,
    /// - `false` else.
    ///
    /// # Panics
    ///
    /// The function will panic if *I* is not 0, 1 or 2.
    ///
    #[must_use = "unused return value"]
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdType) -> bool {
        self.beta::<I>(dart_id) == NULL_DART_ID
    }

    /// Check if a given dart is `i`-free, for all `i`.
    ///
    /// # Return
    ///
    /// Return a boolean indicating if the dart is 0-free, 1-free **and** 2-free.
    #[must_use = "unused return value"]
    pub fn is_free(&self, dart_id: DartIdType) -> bool {
        self.beta::<0>(dart_id) == NULL_DART_ID
            && self.beta::<1>(dart_id) == NULL_DART_ID
            && self.beta::<2>(dart_id) == NULL_DART_ID
    }
}

/// **I-cell-related methods**
impl<T: CoordsFloat> CMap2<T> {
    /// Compute the ID of the vertex a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 0-cell orbit.
    #[must_use = "unused return value"]
    pub fn vertex_id(&self, dart_id: DartIdType) -> VertexIdType {
        atomically(|trans| self.vertex_id_transac(trans, dart_id))
    }

    /// Compute the ID of the vertex a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 0-cell orbit.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn vertex_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<VertexIdType> {
        // min encountered / current dart
        let mut min = dart_id;
        let mut marked = HashSet::new();
        marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit
        marked.insert(dart_id); // we're starting here, so we mark it beforehand
        let mut pending = VecDeque::from([dart_id]);

        while let Some(d) = pending.pop_front() {
            // THIS CODE IS ONLY VALID IN 2D
            let (b2d, b0d) = (
                self.beta_transac::<2>(trans, d)?,
                self.beta_transac::<0>(trans, d)?,
            );
            let image1 = self.beta_transac::<1>(trans, b2d)?;
            if marked.insert(image1) {
                // if true, we did not see this dart yet
                // i.e. we need to visit it later
                min = min.min(image1);
                pending.push_back(image1);
            }
            let image2 = self.beta_transac::<2>(trans, b0d)?;
            if marked.insert(image2) {
                // if true, we did not see this dart yet
                // i.e. we need to visit it later
                min = min.min(image2);
                pending.push_back(image2);
            }
        }

        Ok(min)
    }

    /// Compute the ID of the edge a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 1-cell orbit.
    #[must_use = "unused return value"]
    pub fn edge_id(&self, dart_id: DartIdType) -> EdgeIdType {
        atomically(|trans| self.edge_id_transac(trans, dart_id))
    }

    /// Compute the ID of the edge a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 1-cell orbit.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn edge_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<EdgeIdType> {
        // optimizing this one bc I'm tired
        let b2 = self.beta_transac::<2>(trans, dart_id)?;
        if b2 == NULL_DART_ID {
            Ok(dart_id as EdgeIdType)
        } else {
            Ok(b2.min(dart_id) as EdgeIdType)
        }
    }

    /// Compute the ID of the face a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 2-cell orbit.
    #[must_use = "unused return value"]
    pub fn face_id(&self, dart_id: DartIdType) -> FaceIdType {
        atomically(|trans| self.face_id_transac(trans, dart_id))
    }

    /// Compute the ID of the face a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 2-cell orbit.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn face_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<FaceIdType> {
        // min encountered / current dart
        let mut min = dart_id;
        let mut marked = HashSet::new();
        marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit
        marked.insert(dart_id); // we're starting here, so we mark it beforehand
        let mut pending = VecDeque::from([dart_id]);

        while let Some(d) = pending.pop_front() {
            // THIS CODE IS ONLY VALID IN 2D
            let image1 = self.beta_transac::<1>(trans, d)?;
            if marked.insert(image1) {
                // if true, we did not see this dart yet
                // i.e. we need to visit it later
                min = min.min(image1);
                pending.push_back(image1);
            }
            let image2 = self.beta_transac::<0>(trans, d)?;
            if marked.insert(image2) {
                // if true, we did not see this dart yet
                // i.e. we need to visit it later
                min = min.min(image2);
                pending.push_back(image2);
            }
        }

        Ok(min)
    }

    /// Return the orbit defined by a dart and its `I`-cell.
    ///
    /// # Usage
    ///
    /// The [`Orbit2`] can be iterated upon to retrieve all dart member of the cell. Note that
    /// **the dart passed as an argument is included as the first element of the returned orbit**.
    ///
    /// # Panics
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    #[must_use = "unused return value"]
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdType) -> Orbit2<T> {
        assert!(I < 3);
        match I {
            0 => Orbit2::<'_, T>::new(self, OrbitPolicy::Vertex, dart_id),
            1 => Orbit2::<'_, T>::new(self, OrbitPolicy::Edge, dart_id),
            2 => Orbit2::<'_, T>::new(self, OrbitPolicy::Face, dart_id),
            _ => unreachable!(),
        }
    }

    /// Return an iterator over IDs of all the map's vertices.
    #[must_use = "unused return value"]
    pub fn iter_vertices(&self) -> impl Iterator<Item = VertexIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(
                |(d, unused)| {
                    if unused.read_atomic() {
                        None
                    } else {
                        Some(d)
                    }
                },
            )
            .filter_map(|d| {
                let vid = self.vertex_id(d);
                if d == vid {
                    Some(vid)
                } else {
                    None
                }
            })
    }

    /// Return an iterator over IDs of all the map's edges.
    #[must_use = "unused return value"]
    pub fn iter_edges(&self) -> impl Iterator<Item = EdgeIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(
                |(d, unused)| {
                    if unused.read_atomic() {
                        None
                    } else {
                        Some(d)
                    }
                },
            )
            .filter_map(|d| {
                let eid = self.edge_id(d);
                if d == eid {
                    Some(eid)
                } else {
                    None
                }
            })
    }

    /// Return an iterator over IDs of all the map's faces.
    #[must_use = "unused return value"]
    pub fn iter_faces(&self) -> impl Iterator<Item = FaceIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(
                |(d, unused)| {
                    if unused.read_atomic() {
                        None
                    } else {
                        Some(d)
                    }
                },
            )
            .filter_map(|d| {
                let fid = self.face_id(d);
                if d == fid {
                    Some(fid)
                } else {
                    None
                }
            })
    }
}
