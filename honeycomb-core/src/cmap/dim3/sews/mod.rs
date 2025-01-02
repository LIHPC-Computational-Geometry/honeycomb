mod one;
mod three;
mod two;

use stm::Transaction;

use crate::{
    cmap::{CMap3, CMapResult, DartIdType},
    prelude::CoordsFloat,
};

/// Sew operations
impl<T: CoordsFloat> CMap3<T> {
    pub fn sew<const I: u8>(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> CMapResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.one_sew(trans, ld, rd),
            2 => self.two_sew(trans, ld, rd),
            3 => self.three_sew(trans, ld, rd),
            _ => unreachable!(),
        }
    }

    pub fn unsew<const I: u8>(&self, trans: &mut Transaction, ld: DartIdType) -> CMapResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.one_unsew(trans, ld),
            2 => self.one_unsew(trans, ld),
            3 => self.one_unsew(trans, ld),
            _ => unreachable!(),
        }
    }

    pub fn force_sew<const I: u8>(&self, ld: DartIdType, rd: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_sew(ld, rd),
            2 => self.force_two_sew(ld, rd),
            3 => self.force_three_sew(ld, rd),
            _ => unreachable!(),
        }
    }

    pub fn force_unsew<const I: u8>(&self, ld: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_unsew(ld),
            2 => self.force_two_unsew(ld),
            3 => self.force_three_unsew(ld),
            _ => unreachable!(),
        }
    }
}
