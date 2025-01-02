//! 2D sew implementations

use stm::{atomically, Transaction};

use crate::{
    cmap::{CMap3, CMapResult, DartIdType},
    prelude::CoordsFloat,
};

/// 2-sews
impl<T: CoordsFloat> CMap3<T> {
    /// 2-sew transactional operation.
    pub(crate) fn two_sew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> CMapResult<()> {
        todo!()
    }

    /// 2-sew operation.
    pub(crate) fn force_two_sew(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        todo!()
    }
}

/// 2-unsews
impl<T: CoordsFloat> CMap3<T> {
    /// 2-unsew transactional operation.
    pub(crate) fn two_unsew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> CMapResult<()> {
        todo!()
    }

    /// 2-unsew operation.
    pub(crate) fn force_two_unsew(&self, lhs_dart_id: DartIdType) {
        todo!()
    }
}
