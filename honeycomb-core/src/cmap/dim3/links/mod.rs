mod one;
mod three;
mod two;

use stm::{StmResult, Transaction};

use crate::{
    cmap::{CMap3, DartIdType},
    prelude::CoordsFloat,
};

/// Link operations
impl<T: CoordsFloat> CMap3<T> {
    pub fn link<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.one_link(trans, lhs_dart_id, rhs_dart_id),
            2 => self.two_link(trans, lhs_dart_id, rhs_dart_id),
            3 => self.three_link(trans, lhs_dart_id, rhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn unlink<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> StmResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.one_unlink(trans, lhs_dart_id),
            2 => self.two_unlink(trans, lhs_dart_id),
            3 => self.three_unlink(trans, lhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn force_link<const I: u8>(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_link(lhs_dart_id, rhs_dart_id),
            2 => self.force_two_link(lhs_dart_id, rhs_dart_id),
            3 => self.force_three_link(lhs_dart_id, rhs_dart_id),
            _ => unreachable!(),
        }
    }

    pub fn force_unlink<const I: u8>(&self, lhs_dart_id: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_unlink(lhs_dart_id),
            2 => self.force_two_unlink(lhs_dart_id),
            3 => self.force_three_unlink(lhs_dart_id),
            _ => unreachable!(),
        }
    }
}
