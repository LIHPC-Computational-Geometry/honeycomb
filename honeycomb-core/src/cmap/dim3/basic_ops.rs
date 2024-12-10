//! Basic operations implementation
//!
//! This module contains code used to implement basic operations of combinatorial maps, such as
//! (but not limited to):
//!
//! - Dart addition / insertion / removal
//! - Beta function interfaces
//! - i-cell computations

// ------ IMPORTS

use itertools::Itertools;
use std::collections::{HashSet, VecDeque};
use stm::{atomically, StmError, StmResult, Transaction};

use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{
        CMap3, DartIdType, EdgeIdType, FaceIdType, Orbit3, OrbitPolicy, VertexIdType, VolumeIdType,
        NULL_DART_ID,
    },
    geometry::CoordsFloat,
};

// ------ CONTENT

/// **Dart-related methods**
impl<T: CoordsFloat> CMap3<T> {
    // --- read

    /// Return information about the current number of darts.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_darts(&self) -> usize {
        self.unused_darts.len()
    }

    /// Return information about the current number of unused darts.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_unused_darts(&self) -> usize {
        self.unused_darts.iter().filter(|v| v.read_atomic()).count()
    }

    // --- edit

    pub fn add_free_dart(&mut self) -> DartIdType {
        let new_id = self.n_darts() as DartIdType;
        self.betas.extend(1);
        self.unused_darts.extend(1);
        self.vertices.extend(1);
        self.attributes.extend_storages(1);
        new_id
    }

    pub fn add_free_darts(&mut self, n_darts: usize) -> DartIdType {
        let new_id = self.n_darts() as DartIdType;
        self.betas.extend(n_darts);
        self.unused_darts.extend(n_darts);
        self.vertices.extend(n_darts);
        self.attributes.extend_storages(n_darts);
        new_id
    }

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

    pub fn beta_transac<const I: u8>(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> StmResult<DartIdType> {
        assert!(I < 4);
        self.betas[(I, dart_id)].read(trans)
    }

    pub fn beta_rt_transac(
        &self,
        trans: &mut Transaction,
        i: u8,
        dart_id: DartIdType,
    ) -> StmResult<DartIdType> {
        assert!(i < 4);
        match i {
            0 => self.beta_transac::<0>(trans, dart_id),
            1 => self.beta_transac::<1>(trans, dart_id),
            2 => self.beta_transac::<2>(trans, dart_id),
            3 => self.beta_transac::<3>(trans, dart_id),
            _ => unreachable!(),
        }
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn beta<const I: u8>(&self, dart_id: DartIdType) -> DartIdType {
        assert!(I < 4);
        self.betas[(I, dart_id)].read_atomic()
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn beta_runtime(&self, i: u8, dart_id: DartIdType) -> DartIdType {
        assert!(i < 4);
        match i {
            0 => self.beta::<0>(dart_id),
            1 => self.beta::<1>(dart_id),
            2 => self.beta::<2>(dart_id),
            3 => self.beta::<3>(dart_id),
            _ => unreachable!(),
        }
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdType) -> bool {
        self.beta::<I>(dart_id) == NULL_DART_ID
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn is_free(&self, dart_id: DartIdType) -> bool {
        self.beta::<0>(dart_id) == NULL_DART_ID
            && self.beta::<1>(dart_id) == NULL_DART_ID
            && self.beta::<2>(dart_id) == NULL_DART_ID
            && self.beta::<3>(dart_id) == NULL_DART_ID
    }
}

/// **I-cell-related methods**
impl<T: CoordsFloat> CMap3<T> {
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn vertex_id(&self, dart_id: DartIdType) -> VertexIdType {
        let mut marked = HashSet::new();
        let mut queue = VecDeque::new();
        marked.insert(NULL_DART_ID);
        queue.push_front(dart_id);
        let mut min = dart_id;

        while let Some(d) = queue.pop_front() {
            if marked.insert(d) {
                min = min.min(d);
                let (b0, b2, b3) = (
                    self.beta::<0>(d), // ?
                    self.beta::<2>(d),
                    self.beta::<3>(d),
                );
                queue.push_back(self.beta::<1>(b3));
                queue.push_back(self.beta::<3>(b2));
                queue.push_back(self.beta::<3>(b0)); // ?
            }
        }

        min
    }

    pub fn vertex_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<VertexIdType, StmError> {
        let mut marked = HashSet::new();
        let mut queue = VecDeque::new();
        marked.insert(NULL_DART_ID);
        queue.push_front(dart_id);
        let mut min = dart_id;

        while let Some(d) = queue.pop_front() {
            if marked.insert(d) {
                min = min.min(d);
                let (b0, b2, b3) = (
                    self.beta_transac::<0>(trans, d)?, // ?
                    self.beta_transac::<2>(trans, d)?,
                    self.beta_transac::<3>(trans, d)?,
                );
                queue.push_back(self.beta_transac::<1>(trans, b3)?);
                queue.push_back(self.beta_transac::<3>(trans, b2)?);
                queue.push_back(self.beta_transac::<3>(trans, b0)?); // ?
            }
        }

        Ok(min)
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn edge_id(&self, dart_id: DartIdType) -> EdgeIdType {
        let mut marked = HashSet::new();
        marked.insert(NULL_DART_ID);
        let (mut lb, mut rb) = (dart_id, self.beta::<3>(dart_id));
        let mut min = lb.min(rb);
        let mut alt = true;

        while marked.insert(lb) || marked.insert(rb) {
            (lb, rb) = if alt {
                (self.beta::<2>(lb), self.beta::<2>(rb))
            } else {
                (self.beta::<3>(lb), self.beta::<3>(rb))
            };
            if lb != NULL_DART_ID {
                min = min.min(lb);
            }
            if rb != NULL_DART_ID {
                min = min.min(rb);
            }
            alt = !alt;
        }

        min
    }

    pub fn edge_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<EdgeIdType, StmError> {
        let mut marked = HashSet::new();
        marked.insert(NULL_DART_ID);
        let (mut lb, mut rb) = (dart_id, self.beta_transac::<3>(trans, dart_id)?);
        let mut min = lb.min(rb);
        let mut alt = true;

        while marked.insert(lb) || marked.insert(rb) {
            (lb, rb) = if alt {
                (
                    self.beta_transac::<2>(trans, lb)?,
                    self.beta_transac::<2>(trans, rb)?,
                )
            } else {
                (
                    self.beta_transac::<3>(trans, lb)?,
                    self.beta_transac::<3>(trans, rb)?,
                )
            };
            if lb != NULL_DART_ID {
                min = min.min(lb);
            }
            if rb != NULL_DART_ID {
                min = min.min(rb);
            }
            alt = !alt;
        }

        Ok(min)
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn face_id(&self, dart_id: DartIdType) -> FaceIdType {
        let mut marked = HashSet::new();
        marked.insert(NULL_DART_ID);
        let b3_dart_id = self.beta::<3>(dart_id);
        let (mut lb, mut rb) = (dart_id, b3_dart_id);
        let mut min = lb.min(rb);

        while marked.insert(lb) || marked.insert(rb) {
            (lb, rb) = (self.beta::<1>(lb), self.beta::<0>(rb));
            if lb != NULL_DART_ID {
                min = min.min(lb);
            }
            if rb != NULL_DART_ID {
                min = min.min(rb);
            }
        }
        // face is open, we need to iterate in the other direction
        if lb == NULL_DART_ID || rb == NULL_DART_ID {
            (lb, rb) = (self.beta::<0>(dart_id), self.beta::<1>(b3_dart_id));
            while marked.insert(lb) || marked.insert(rb) {
                (lb, rb) = (self.beta::<0>(lb), self.beta::<1>(rb));
                if lb != NULL_DART_ID {
                    min = min.min(lb);
                }
                if rb != NULL_DART_ID {
                    min = min.min(rb);
                }
            }
        }

        min
    }

    pub fn face_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<FaceIdType, StmError> {
        let mut marked = HashSet::new();
        marked.insert(NULL_DART_ID);
        let b3_dart_id = self.beta_transac::<3>(trans, dart_id)?;
        let (mut lb, mut rb) = (dart_id, b3_dart_id);
        let mut min = lb.min(rb);

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
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn volume_id(&self, dart_id: DartIdType) -> VolumeIdType {
        let mut marked = HashSet::new();
        let mut queue = VecDeque::new();
        marked.insert(NULL_DART_ID);
        queue.push_front(dart_id);
        let mut min = dart_id;

        while let Some(d) = queue.pop_front() {
            if marked.insert(d) {
                min = min.min(d);
                queue.push_back(self.beta::<1>(d));
                queue.push_back(self.beta::<0>(d)); // ?
                queue.push_back(self.beta::<2>(d));
            }
        }

        min
    }

    pub fn volume_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<VolumeIdType, StmError> {
        let mut marked = HashSet::new();
        let mut queue = VecDeque::new();
        marked.insert(NULL_DART_ID);
        queue.push_front(dart_id);
        let mut min = dart_id;

        while let Some(d) = queue.pop_front() {
            if marked.insert(d) {
                min = min.min(d);
                queue.push_back(self.beta_transac::<1>(trans, d)?);
                queue.push_back(self.beta_transac::<0>(trans, d)?); // ?
                queue.push_back(self.beta_transac::<2>(trans, d)?);
            }
        }

        Ok(min)
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdType) -> Orbit3<'_, T> {
        assert!(I < 4);
        match I {
            0 => Orbit3::new(self, OrbitPolicy::Vertex, dart_id),
            1 => Orbit3::new(self, OrbitPolicy::Edge, dart_id),
            2 => Orbit3::new(self, OrbitPolicy::Face, dart_id),
            3 => todo!(),
            _ => unreachable!(),
        }
    }

    pub fn fetch_vertices(&self) -> impl Iterator<Item = VertexIdType> + '_ {
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

    pub fn fetch_edges(&self) -> impl Iterator<Item = EdgeIdType> + '_ {
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

    pub fn fetch_faces(&self) -> impl Iterator<Item = FaceIdType> + '_ {
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

    pub fn fetch_volumes(&self) -> impl Iterator<Item = VolumeIdType> + '_ {
        (1..self.n_darts() as DartIdType)
            .zip(self.unused_darts.iter().skip(1))
            .filter_map(|(d, unused)| {
                if unused.read_atomic() {
                    None
                } else {
                    Some(self.volume_id(d))
                }
            })
            .unique()
    }
}
