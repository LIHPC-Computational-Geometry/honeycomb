//! Basic operations implementation
//!
//! This module contains code used to implement basic operations of combinatorial maps, such as
//! (but not limited to):
//!
//! - Dart addition / insertion / removal
//! - Beta function interfaces
//! - i-cell computations

use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};

use rayon::prelude::*;

use crate::cmap::{CMap2, DartIdType, EdgeIdType, FaceIdType, NULL_DART_ID, VertexIdType};
use crate::geometry::CoordsFloat;
use crate::stm::{StmClosureResult, Transaction, atomically};

// use thread local hashset and queue for orbit traversal of ID comp.
// not applied to orbit currently bc they are lazily onsumed, and therefore require dedicated
// instances to be robust
thread_local! {
    static AUXILIARIES: RefCell<(VecDeque<DartIdType>, HashSet<DartIdType>)> = RefCell::new((VecDeque::with_capacity(10), HashSet::with_capacity(10)));
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
    pub fn beta_tx<const I: u8>(
        &self,
        t: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<DartIdType> {
        assert!(I < 3);
        self.betas[(I, dart_id)].read(t)
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
    pub fn beta_rt_tx(
        &self,
        t: &mut Transaction,
        i: u8,
        dart_id: DartIdType,
    ) -> StmClosureResult<DartIdType> {
        assert!(i < 3);
        match i {
            0 => self.beta_tx::<0>(t, dart_id),
            1 => self.beta_tx::<1>(t, dart_id),
            2 => self.beta_tx::<2>(t, dart_id),
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
        atomically(|t| self.is_free_tx(t, dart_id))
    }

    #[allow(clippy::missing_errors_doc)]
    /// Check if a given dart is `i`-free, for all `i`.
    ///
    /// # Return / Errors
    ///
    /// Return a boolean indicating if the dart is 0-free, 1-free **and** 2-free.
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    #[must_use = "unused return value"]
    pub fn is_free_tx(&self, t: &mut Transaction, dart_id: DartIdType) -> StmClosureResult<bool> {
        Ok(self.beta_tx::<0>(t, dart_id)? == NULL_DART_ID
            && self.beta_tx::<1>(t, dart_id)? == NULL_DART_ID
            && self.beta_tx::<2>(t, dart_id)? == NULL_DART_ID)
    }
}

/// **I-cell-related methods**
impl<T: CoordsFloat> CMap2<T> {
    /// Compute the ID of the vertex a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 0-cell orbit.
    #[must_use = "unused return value"]
    pub fn vertex_id(&self, dart_id: DartIdType) -> VertexIdType {
        atomically(|t| self.vertex_id_tx(t, dart_id))
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
    pub fn vertex_id_tx(
        &self,
        t: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<VertexIdType> {
        AUXILIARIES.with(|cell| {
            let (pending, marked) = &mut *cell.borrow_mut();
            // clear from previous computations
            pending.clear();
            marked.clear();
            // initialize
            pending.push_back(dart_id);
            marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit
            marked.insert(dart_id); // we're starting here, so we mark it beforehand

            let mut min = dart_id;

            while let Some(d) = pending.pop_front() {
                // THIS CODE IS ONLY VALID IN 2D
                let (b2d, b0d) = (self.beta_tx::<2>(t, d)?, self.beta_tx::<0>(t, d)?);
                let image1 = self.beta_tx::<1>(t, b2d)?;
                if marked.insert(image1) {
                    // if true, we did not see this dart yet
                    // i.e. we need to visit it later
                    min = min.min(image1);
                    pending.push_back(image1);
                }
                let image2 = self.beta_tx::<2>(t, b0d)?;
                if marked.insert(image2) {
                    // if true, we did not see this dart yet
                    // i.e. we need to visit it later
                    min = min.min(image2);
                    pending.push_back(image2);
                }
            }

            Ok(min)
        })
    }

    /// Compute the ID of the edge a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 1-cell orbit.
    #[must_use = "unused return value"]
    pub fn edge_id(&self, dart_id: DartIdType) -> EdgeIdType {
        atomically(|t| self.edge_id_tx(t, dart_id))
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
    pub fn edge_id_tx(
        &self,
        t: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<EdgeIdType> {
        // optimizing this one bc I'm tired
        let b2 = self.beta_tx::<2>(t, dart_id)?;
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
        atomically(|t| self.face_id_tx(t, dart_id))
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
    pub fn face_id_tx(
        &self,
        t: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<FaceIdType> {
        AUXILIARIES.with(|cell| {
            let (pending, marked) = &mut *cell.borrow_mut();
            // clear from previous computations
            pending.clear();
            marked.clear();
            // initialize
            pending.push_back(dart_id);
            marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit
            marked.insert(dart_id); // we're starting here, so we mark it beforehand

            let mut min = dart_id;

            while let Some(d) = pending.pop_front() {
                // THIS CODE IS ONLY VALID IN 2D
                let image1 = self.beta_tx::<1>(t, d)?;
                if marked.insert(image1) {
                    // if true, we did not see this dart yet
                    // i.e. we need to visit it later
                    min = min.min(image1);
                    pending.push_back(image1);
                }
                let image2 = self.beta_tx::<0>(t, d)?;
                if marked.insert(image2) {
                    // if true, we did not see this dart yet
                    // i.e. we need to visit it later
                    min = min.min(image2);
                    pending.push_back(image2);
                }
            }

            Ok(min)
        })
    }

    /// Return an iterator over IDs of all the map's vertices.
    #[must_use = "unused return value"]
    pub fn iter_vertices(&self) -> impl Iterator<Item = VertexIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(
                |(d, unused)| {
                    if unused.read_atomic() { None } else { Some(d) }
                },
            )
            .filter_map(|d| {
                let vid = self.vertex_id(d);
                if d == vid { Some(vid) } else { None }
            })
    }

    /// Return an iterator over IDs of all the map's edges.
    #[must_use = "unused return value"]
    pub fn iter_edges(&self) -> impl Iterator<Item = EdgeIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(
                |(d, unused)| {
                    if unused.read_atomic() { None } else { Some(d) }
                },
            )
            .filter_map(|d| {
                let eid = self.edge_id(d);
                if d == eid { Some(eid) } else { None }
            })
    }

    /// Return an iterator over IDs of all the map's faces.
    #[must_use = "unused return value"]
    pub fn iter_faces(&self) -> impl Iterator<Item = FaceIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(
                |(d, unused)| {
                    if unused.read_atomic() { None } else { Some(d) }
                },
            )
            .filter_map(|d| {
                let fid = self.face_id(d);
                if d == fid { Some(fid) } else { None }
            })
    }

    /// Return an iterator over IDs of all the map's vertices.
    #[must_use = "unused return value"]
    pub fn par_iter_vertices(&self) -> impl ParallelIterator<Item = VertexIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .into_par_iter()
            .filter_map(|d| if self.is_unused(d) { None } else { Some(d) })
            .filter_map(|d| {
                let vid = self.vertex_id(d);
                if d == vid { Some(vid) } else { None }
            })
    }

    /// Return an iterator over IDs of all the map's edges.
    #[must_use = "unused return value"]
    pub fn par_iter_edges(&self) -> impl ParallelIterator<Item = EdgeIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .into_par_iter()
            .filter_map(|d| if self.is_unused(d) { None } else { Some(d) })
            .filter_map(|d| {
                let eid = self.edge_id(d);
                if d == eid { Some(eid) } else { None }
            })
    }

    /// Return an iterator over IDs of all the map's faces.
    #[must_use = "unused return value"]
    pub fn par_iter_faces(&self) -> impl ParallelIterator<Item = FaceIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .into_par_iter()
            .filter_map(|d| if self.is_unused(d) { None } else { Some(d) })
            .filter_map(|d| {
                let fid = self.face_id(d);
                if d == fid { Some(fid) } else { None }
            })
    }
}
