mod one;
mod two;

use stm::{StmResult, Transaction};

use crate::{
    cmap::{CMap2, CMapResult, DartIdType},
    prelude::CoordsFloat,
};

/// Sew operations
impl<T: CoordsFloat> CMap2<T> {
    pub fn sew<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.one_sew(trans, lhs_dart_id, rhs_dart_id),
            2 => self.two_sew(trans, lhs_dart_id, rhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn unsew<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.one_unsew(trans, lhs_dart_id),
            2 => self.two_unsew(trans, lhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn force_sew<const I: u8>(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_sew(lhs_dart_id, rhs_dart_id),
            2 => self.force_two_sew(lhs_dart_id, rhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn force_unsew<const I: u8>(&self, lhs_dart_id: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_unsew(lhs_dart_id),
            2 => self.force_two_unsew(lhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn try_sew<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> CMapResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.try_one_sew(trans, lhs_dart_id, rhs_dart_id),
            2 => self.try_two_sew(trans, lhs_dart_id, rhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn try_unsew<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> CMapResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.try_one_unsew(trans, lhs_dart_id),
            2 => self.try_two_unsew(trans, lhs_dart_id),
            _ => unreachable!(),
        }
    }
}
