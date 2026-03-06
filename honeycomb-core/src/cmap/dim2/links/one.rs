use crate::cmap::{CMap2, DartIdType, LinkError};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult};

#[doc(hidden)]
/// 1-links
impl<T: CoordsFloat> CMap2<T> {
    /// 1-link implementation.
    pub(super) fn one_link_tx(
        &self,
        t: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.one_link_core(t, lhs_dart_id, rhs_dart_id)
    }
}

#[doc(hidden)]
/// 1-unlinks
impl<T: CoordsFloat> CMap2<T> {
    /// 1-unlink implementation.
    pub(super) fn one_unlink_tx(
        &self,
        t: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.one_unlink_core(t, lhs_dart_id)
    }
}
