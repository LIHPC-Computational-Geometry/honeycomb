use crate::cmap::{CMap2, DartIdType, LinkError};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, atomically_with_err};

#[doc(hidden)]
/// 1-links
impl<T: CoordsFloat> CMap2<T> {
    /// 1-link implementation.
    pub(super) fn one_link(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id)
    }

    /// 1-link defensive implementation.
    pub(super) fn force_one_link(
        &self,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id))
    }
}

#[doc(hidden)]
/// 1-unlinks
impl<T: CoordsFloat> CMap2<T> {
    /// 1-unlink implementation.
    pub(super) fn one_unlink(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.one_unlink_core(trans, lhs_dart_id)
    }

    /// 1-unlink defensive implementation.
    pub(super) fn force_one_unlink(&self, lhs_dart_id: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.betas.one_unlink_core(trans, lhs_dart_id))
    }
}
