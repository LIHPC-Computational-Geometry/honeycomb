//! 2D link implementations

use crate::{
    cmap::{CMap3, DartIdType, LinkError},
    prelude::CoordsFloat,
    stm::{atomically_with_err, Transaction, TransactionClosureResult},
};

/// 2-links
impl<T: CoordsFloat> CMap3<T> {
    /// 2-link operation.
    pub(crate) fn two_link(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)
    }

    /// 2-link operation.
    pub(crate) fn force_two_link(
        &self,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id))
    }
}

/// 2-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 2-unlink operation.
    pub(crate) fn two_unlink(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.two_unlink_core(trans, lhs_dart_id)
    }

    /// 2-unlink operation.
    pub(crate) fn force_two_unlink(&self, lhs_dart_id: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.betas.two_unlink_core(trans, lhs_dart_id))
    }
}
