//! 3D link implementations

use stm::{atomically, StmResult, Transaction};

use crate::{
    cmap::{CMap3, DartIdType},
    prelude::CoordsFloat,
};

/// 3-links
impl<T: CoordsFloat> CMap3<T> {
    /// 3-link operation.
    pub(crate) fn three_link(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        todo!()
    }

    /// 3-link operation.
    pub(crate) fn force_three_link(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        todo!()
    }
}

/// 3-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 3-unlink operation.
    pub(crate) fn three_unlink(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        todo!()
    }

    /// 3-unlink operation.
    pub(crate) fn force_three_unlink(&self, lhs_dart_id: DartIdType) {
        todo!()
    }
}
