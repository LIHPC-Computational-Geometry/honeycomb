use crate::cmap::{CMap2, DartIdType, LinkError};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, atomically_with_err};

#[doc(hidden)]
/// 2-links
impl<T: CoordsFloat> CMap2<T> {
    /// 2-link implementation.
    pub(super) fn two_link(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)
    }

    /// 2-link defensive implementation.
    pub(super) fn force_two_link(
        &self,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id))
    }
}

#[doc(hidden)]
/// 2-unlinks
impl<T: CoordsFloat> CMap2<T> {
    /// 2-unlink implementation.
    pub(super) fn two_unlink(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.two_unlink_core(trans, lhs_dart_id)
    }

    /// 2-unlink defensive implementation.
    pub(super) fn force_two_unlink(&self, lhs_dart_id: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.betas.two_unlink_core(trans, lhs_dart_id))
    }
}
