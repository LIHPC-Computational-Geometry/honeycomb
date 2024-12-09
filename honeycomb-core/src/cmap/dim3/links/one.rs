//! 1D link implementations

use stm::{atomically, StmResult, Transaction};

use crate::{
    cmap::{CMap3, DartIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

/// 1-links
impl<T: CoordsFloat> CMap3<T> {
    /// 1-link operation.
    pub(crate) fn one_link(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> StmResult<()> {
        self.betas.one_link_core(trans, ld, rd)?;
        let (b3_ld, b3_rd) = (
            self.beta_transac::<3>(trans, ld)?,
            self.beta_transac::<3>(trans, rd)?,
        );
        if b3_ld != NULL_DART_ID && b3_rd != NULL_DART_ID {
            self.betas.one_link_core(trans, b3_rd, b3_ld)?;
        }
        Ok(())
    }

    /// 1-link operation.
    ///
    /// This variant is equivalent to `one_link`, but internally uses a transaction that will be
    /// retried until validated.
    pub(crate) fn force_one_link(&self, ld: DartIdType, rd: DartIdType) {
        atomically(|trans| self.one_link(trans, ld, rd));
    }
}

/// 1-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 1-unlink operation.
    pub(crate) fn one_unlink(&self, trans: &mut Transaction, ld: DartIdType) -> StmResult<()> {
        let rd = self.beta_transac::<1>(trans, ld)?;
        self.betas.one_unlink_core(trans, ld)?;
        let (b3_ld, b3_rd) = (
            self.beta_transac::<3>(trans, ld)?,
            self.beta_transac::<3>(trans, rd)?,
        );
        if b3_ld != NULL_DART_ID && b3_rd != NULL_DART_ID {
            assert!(self.beta_transac::<1>(trans, b3_rd)? == b3_ld);
            self.betas.one_unlink_core(trans, b3_rd)?;
        }
        Ok(())
    }

    /// 1-unlink operation.
    ///
    /// This variant is equivalent to `one_unlink`, but internally uses a transaction that will be
    /// retried until validated.
    pub(crate) fn force_one_unlink(&self, ld: DartIdType) {
        atomically(|trans| self.one_unlink(trans, ld));
    }
}
