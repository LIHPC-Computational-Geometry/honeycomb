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

use crate::attributes::UnknownAttributeStorage;
use crate::cmap::{
    CMap3, DartIdType, EdgeIdType, FaceIdType, NULL_DART_ID, VertexIdType, VolumeIdType,
};
use crate::geometry::CoordsFloat;
use crate::stm::{StmClosureResult, StmError, Transaction, atomically};

// use thread local hashset and queue for orbit traversal of ID comp.
// not applied to orbit currently bc they are lazily onsumed, and therefore require dedicated
// instances to be robust
thread_local! {
    static AUXILIARIES: RefCell<(VecDeque<DartIdType>, HashSet<DartIdType>)> = RefCell::new((VecDeque::with_capacity(10), HashSet::with_capacity(10)));
}

/// **Dart-related methods**
impl<T: CoordsFloat> CMap3<T> {
    // --- read

    /// Return the current number of darts.
    #[must_use = "unused return value"]
    pub fn n_darts(&self) -> usize {
        self.unused_darts.len()
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
    /// Returns the ID of the new dart.
    pub fn add_free_dart(&mut self) -> DartIdType {
        let new_id = self.n_darts() as DartIdType;
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
    /// Returns the ID of the first new dart. Other IDs are in the range `ID..ID+n_darts`.
    pub fn add_free_darts(&mut self, n_darts: usize) -> DartIdType {
        let new_id = self.n_darts() as DartIdType;
        self.betas.extend(n_darts);
        self.unused_darts.extend(n_darts);
        self.vertices.extend(n_darts);
        self.attributes.extend_storages(n_darts);
        new_id
    }

    /// Insert a new free dart into the map.
    ///
    /// This method attempts to reuse an unused dart slot if available; otherwise, it adds a new one.
    ///
    /// # Return
    ///
    /// Returns the ID of the inserted dart.
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
    /// The removed dart identifier is added to the list of free darts. This way of proceeding is
    /// necessary as the structure relies on dart indexing for encoding data, making reordering of
    /// any sort extremely costly.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdType` -- Identifier of the dart to remove.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - the dart is not free for all *i*,
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
impl<T: CoordsFloat> CMap3<T> {
    // --- read

    /// Return β<sub>`I`</sub>(`dart_id`).
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method will panic if `I` is not 0, 1, 2, or 3.
    pub fn beta_transac<const I: u8>(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmClosureResult<DartIdType> {
        assert!(I < 4);
        self.betas[(I, dart_id)].read(trans)
    }

    /// Return β<sub>`i`</sub>(`dart_id`).
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method will panic if `i` is not 0, 1, 2, or 3.
    pub fn beta_rt_transac(
        &self,
        trans: &mut Transaction,
        i: u8,
        dart_id: DartIdType,
    ) -> StmClosureResult<DartIdType> {
        assert!(i < 4);
        match i {
            0 => self.beta_transac::<0>(trans, dart_id),
            1 => self.beta_transac::<1>(trans, dart_id),
            2 => self.beta_transac::<2>(trans, dart_id),
            3 => self.beta_transac::<3>(trans, dart_id),
            _ => unreachable!(),
        }
    }

    /// Return β<sub>`I`</sub>(`dart_id`).
    ///
    /// # Panics
    ///
    /// The method will panic if `I` is not 0, 1, 2, or 3.
    #[must_use = "unused return value"]
    pub fn beta<const I: u8>(&self, dart_id: DartIdType) -> DartIdType {
        assert!(I < 4);
        self.betas[(I, dart_id)].read_atomic()
    }

    /// Return β<sub>`i`</sub>(`dart_id`).
    ///
    /// # Panics
    ///
    /// The method will panic if `i` is not 0, 1, 2, or 3.
    #[must_use = "unused return value"]
    pub fn beta_rt(&self, i: u8, dart_id: DartIdType) -> DartIdType {
        assert!(i < 4);
        match i {
            0 => self.beta::<0>(dart_id),
            1 => self.beta::<1>(dart_id),
            2 => self.beta::<2>(dart_id),
            3 => self.beta::<3>(dart_id),
            _ => unreachable!(),
        }
    }

    /// Check if a given dart is `I`-free.
    ///
    /// # Return
    ///
    /// The method returns:
    /// - `true` if β<sub>`I`</sub>(`dart_id`) = `NULL_DART_ID`,
    /// - `false` otherwise.
    ///
    /// # Panics
    ///
    /// The function will panic if *I* is not 0, 1, 2, or 3.
    #[must_use = "unused return value"]
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdType) -> bool {
        self.beta::<I>(dart_id) == NULL_DART_ID
    }

    /// Check if a given dart is free for all `i`.
    ///
    /// # Return
    ///
    /// Returns `true` if the dart is 0-free, 1-free, 2-free, **and** 3-free.
    #[must_use = "unused return value"]
    pub fn is_free(&self, dart_id: DartIdType) -> bool {
        self.beta::<0>(dart_id) == NULL_DART_ID
            && self.beta::<1>(dart_id) == NULL_DART_ID
            && self.beta::<2>(dart_id) == NULL_DART_ID
            && self.beta::<3>(dart_id) == NULL_DART_ID
    }
}

/// **I-cell-related methods**
impl<T: CoordsFloat> CMap3<T> {
    /// Compute the ID of the vertex a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 0-cell orbit.
    #[must_use = "unused return value"]
    pub fn vertex_id(&self, dart_id: DartIdType) -> VertexIdType {
        atomically(|t| self.vertex_id_transac(t, dart_id))
    }

    /// Compute the ID of the vertex a given dart is part of, transactionally.
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
    ) -> Result<VertexIdType, StmError> {
        AUXILIARIES.with(|t| {
            let (pending, marked) = &mut *t.borrow_mut();
            // clear from previous computations
            pending.clear();
            marked.clear();
            // initialize
            pending.push_front(dart_id);
            marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit

            let mut min = dart_id;

            while let Some(d) = pending.pop_front() {
                if marked.insert(d) {
                    min = min.min(d);
                    let (b0, b2, b3) = (
                        self.beta_transac::<0>(trans, d)?, // ?
                        self.beta_transac::<2>(trans, d)?,
                        self.beta_transac::<3>(trans, d)?,
                    );
                    pending.push_back(self.beta_transac::<1>(trans, b3)?);
                    pending.push_back(self.beta_transac::<3>(trans, b2)?);
                    pending.push_back(self.beta_transac::<1>(trans, b2)?);
                    pending.push_back(self.beta_transac::<3>(trans, b0)?); // ?
                    pending.push_back(self.beta_transac::<2>(trans, b0)?); // ?
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
        atomically(|t| self.edge_id_transac(t, dart_id))
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
    ) -> Result<EdgeIdType, StmError> {
        AUXILIARIES.with(|t| {
            let (pending, marked) = &mut *t.borrow_mut();
            // clear from previous computations
            pending.clear();
            marked.clear();
            // initialize
            pending.push_back(dart_id);
            marked.insert(NULL_DART_ID);

            let mut min = dart_id;

            while let Some(d) = pending.pop_front() {
                if marked.insert(d) {
                    min = min.min(d);
                    [
                        self.beta_transac::<2>(trans, d)?,
                        self.beta_transac::<3>(trans, d)?,
                    ]
                    .into_iter()
                    .for_each(|im| {
                        pending.push_back(im);
                    });
                }
            }

            Ok(min)
        })
    }

    /// Compute the ID of the face a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 2-cell orbit.
    #[must_use = "unused return value"]
    pub fn face_id(&self, dart_id: DartIdType) -> FaceIdType {
        atomically(|t| self.face_id_transac(t, dart_id))
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
    ) -> Result<FaceIdType, StmError> {
        AUXILIARIES.with(|t| {
            let (_pending, marked) = &mut *t.borrow_mut();
            // clear from previous computations
            marked.clear();
            // initialize
            marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit

            let b3_dart_id = self.beta_transac::<3>(trans, dart_id)?;
            let (mut lb, mut rb) = (dart_id, b3_dart_id);
            let mut min = if rb == NULL_DART_ID { lb } else { lb.min(rb) };

            while marked.insert(lb) || marked.insert(rb) {
                (lb, rb) = (
                    self.beta_transac::<1>(trans, lb)?,
                    self.beta_transac::<0>(trans, rb)?,
                );
                if lb != NULL_DART_ID {
                    min = min.min(lb);
                }
                if rb != NULL_DART_ID {
                    min = min.min(rb);
                }
            }
            // face is open, we need to iterate in the other direction
            if lb == NULL_DART_ID || rb == NULL_DART_ID {
                (lb, rb) = (
                    self.beta_transac::<0>(trans, dart_id)?,
                    self.beta_transac::<1>(trans, b3_dart_id)?,
                );
                while marked.insert(lb) || marked.insert(rb) {
                    (lb, rb) = (
                        self.beta_transac::<0>(trans, lb)?,
                        self.beta_transac::<1>(trans, rb)?,
                    );
                    if lb != NULL_DART_ID {
                        min = min.min(lb);
                    }
                    if rb != NULL_DART_ID {
                        min = min.min(rb);
                    }
                }
            }

            Ok(min)
        })
    }

    /// Compute the ID of the volume a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 3-cell orbit.
    #[must_use = "unused return value"]
    pub fn volume_id(&self, dart_id: DartIdType) -> VolumeIdType {
        atomically(|t| self.volume_id_transac(t, dart_id))
    }

    /// Compute the ID of the volume a given dart is part of.
    ///
    /// This corresponds to the minimum dart ID among darts composing the 3-cell orbit.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    pub fn volume_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<VolumeIdType, StmError> {
        AUXILIARIES.with(|t| {
            let (pending, marked) = &mut *t.borrow_mut();
            // clear from previous computations
            pending.clear();
            marked.clear();
            // initialize
            pending.push_front(dart_id);
            marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit

            let mut min = dart_id;

            while let Some(d) = pending.pop_front() {
                if marked.insert(d) {
                    min = min.min(d);
                    pending.push_back(self.beta_transac::<1>(trans, d)?);
                    pending.push_back(self.beta_transac::<0>(trans, d)?); // ?
                    pending.push_back(self.beta_transac::<2>(trans, d)?);
                }
            }

            Ok(min)
        })
    }

    /// Return an iterator over IDs of all the map's vertices.
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

    /// Return an iterator over IDs of all the map's volumes.
    pub fn iter_volumes(&self) -> impl Iterator<Item = VolumeIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(
                |(d, unused)| {
                    if unused.read_atomic() { None } else { Some(d) }
                },
            )
            .filter_map(|d| {
                let vid = self.volume_id(d);
                if d == vid { Some(vid) } else { None }
            })
    }
}
