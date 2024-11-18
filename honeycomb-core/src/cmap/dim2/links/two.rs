//! 2D link implementations

use stm::{atomically, StmError, Transaction};

use crate::{
    cmap::{CMap2, DartIdType},
    prelude::CoordsFloat,
};

/// 2-links
impl<T: CoordsFloat> CMap2<T> {
    /// 2-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>2</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` or `rhs_dart_id` isn't 2-free.
    pub fn two_link(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)
    }

    /// 2-link operation.
    ///
    /// This variant is equivalent to `two_link`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_two_link(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        atomically(|trans| self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id));
    }
}

/// 2-unlinks
impl<T: CoordsFloat> CMap2<T> {
    /// 2-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>2</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 2-free.
    pub fn two_unlink(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        self.betas.two_unlink_core(trans, lhs_dart_id)
    }

    /// 2-unlink operation.
    ///
    /// This variant is equivalent to `two_unlink`, but internally uses a transaction that will
    /// be retried until validated.
    pub fn force_two_unlink(&self, lhs_dart_id: DartIdType) {
        atomically(|trans| self.betas.two_unlink_core(trans, lhs_dart_id));
    }
}
