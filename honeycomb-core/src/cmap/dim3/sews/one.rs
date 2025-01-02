//! 1D sew implementations

use stm::{atomically, Transaction};

use crate::{
    cmap::{CMap3, CMapResult, DartIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

/// 1-sews
impl<T: CoordsFloat> CMap3<T> {
    /// 1-sew transactional operation.
    pub(crate) fn one_sew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> CMapResult<()> {
        todo!()
    }

    /// 1-sew operation.
    pub(crate) fn force_one_sew(&self, ld: DartIdType, rd: DartIdType) {
        todo!()
    }
}

/// 1-unsews
impl<T: CoordsFloat> CMap3<T> {
    /// 1-unsew transactional operation.
    pub(crate) fn one_unsew(&self, trans: &mut Transaction, ld: DartIdType) -> CMapResult<()> {
        todo!()
    }

    /// 1-unsew operation.
    pub(crate) fn force_one_unsew(&self, ld: DartIdType) {
        todo!()
    }
}
