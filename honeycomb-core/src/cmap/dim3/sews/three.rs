//! 3D sew implementations

use stm::{atomically, Transaction};

use crate::{
    cmap::{CMap3, CMapResult, DartIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

/// 3-sews
impl<T: CoordsFloat> CMap3<T> {
    /// 3-sew operation.
    pub(crate) fn three_sew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> CMapResult<()> {
        todo!()
    }

    /// 3-sew operation.
    pub(crate) fn force_three_sew(&self, ld: DartIdType, rd: DartIdType) {
        todo!()
    }
}

/// 3-unsews
impl<T: CoordsFloat> CMap3<T> {
    /// 3-unsew operation.
    pub(crate) fn three_unsew(&self, trans: &mut Transaction, ld: DartIdType) -> CMapResult<()> {
        todo!()
    }

    /// 3-unsew operation.
    pub(crate) fn force_three_unsew(&self, ld: DartIdType) {
        todo!()
    }
}
