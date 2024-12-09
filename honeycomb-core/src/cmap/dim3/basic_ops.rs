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
    DartIdType, EdgeIdType, FaceIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID,
};
use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{CMap3, EdgeCollection, FaceCollection, VertexCollection},
    geometry::CoordsFloat,
};
use std::collections::{BTreeSet, VecDeque};
use stm::{atomically, StmError, StmResult, Transaction};

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
        todo!()
    }

    pub fn vertex_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<VertexIdType, StmError> {
        todo!()
    }

    pub fn edge_id(&self, dart_id: DartIdType) -> EdgeIdType {
        todo!()
    }

    pub fn edge_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<EdgeIdType, StmError> {
        todo!()
    }

    pub fn face_id(&self, dart_id: DartIdType) -> FaceIdType {
        todo!()
    }

    pub fn face_id_transac(
        &self,
        trans: &mut Transaction,
        dart_id: DartIdType,
    ) -> Result<FaceIdType, StmError> {
        todo!()
    }

    pub fn i_cell<const I: u8>(&self, dart_id: DartIdType) -> Orbit2<T> {
        todo!()
    }

    pub fn fetch_vertices(&self) -> VertexCollection<T> {
        todo!()
    }

    pub fn fetch_edges(&self) -> EdgeCollection<T> {
        todo!()
    }

    pub fn fetch_faces(&self) -> FaceCollection<T> {
        todo!()
    }
}
