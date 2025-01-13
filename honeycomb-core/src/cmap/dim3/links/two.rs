//! 2D link implementations

use stm::{atomically, StmResult, Transaction};

use crate::{
    cmap::{CMap3, DartIdType},
    prelude::CoordsFloat,
};

/// 2-links
impl<T: CoordsFloat> CMap3<T> {
    /// 2-link operation.
    pub(crate) fn two_link(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)
    }

    /// 2-link operation.
    pub(crate) fn force_two_link(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        atomically(|trans| self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id));
    }
}

/// 2-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 2-unlink operation.
    pub(crate) fn two_unlink(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        self.betas.two_unlink_core(trans, lhs_dart_id)
    }

    /// 2-unlink operation.
    pub(crate) fn force_two_unlink(&self, lhs_dart_id: DartIdType) {
        atomically(|trans| self.betas.two_unlink_core(trans, lhs_dart_id));
    }
}
