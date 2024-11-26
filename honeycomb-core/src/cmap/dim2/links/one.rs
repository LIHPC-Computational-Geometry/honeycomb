//! 1D link implementations

use fast_stm::{atomically, StmResult, Transaction};

use crate::{
    cmap::{CMap2, DartIdType},
    prelude::CoordsFloat,
};

/// 1-links
impl<T: CoordsFloat> CMap2<T> {
    /// 1-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>1</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s). The *β<sub>0</sub>* function is also updated.
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
    /// This method may panic if `lhs_dart_id` isn't 1-free or `rhs_dart_id` isn't 0-free.
    pub fn one_link(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id)
    }

    /// 1-link operation.
    ///
    /// This variant is equivalent to `one_link`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_one_link(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        atomically(|trans| self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id));
    }
}

/// 1-unlinks
impl<T: CoordsFloat> CMap2<T> {
    /// 1-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>1</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s). The *β<sub>0</sub>* function is
    /// also updated.
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
    /// This method may panic if one of `lhs_dart_id` is already 1-free.
    pub fn one_unlink(&self, trans: &mut Transaction, lhs_dart_id: DartIdType) -> StmResult<()> {
        self.betas.one_unlink_core(trans, lhs_dart_id)
    }

    /// 1-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>1</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s). The *β<sub>0</sub>* function is
    /// also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 1-free.
    pub fn force_one_unlink(&self, lhs_dart_id: DartIdType) {
        atomically(|trans| self.betas.one_unlink_core(trans, lhs_dart_id));
    }
}
