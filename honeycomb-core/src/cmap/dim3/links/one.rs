//! 1D link implementations

use crate::cmap::{CMap3, DartIdType, LinkError, NULL_DART_ID};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, abort};

/// 1-links
impl<T: CoordsFloat> CMap3<T> {
    /// 1-link operation.
    pub(crate) fn one_link_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.one_link_core(t, ld, rd)?;
        let (b3_ld, b3_rd) = (self.beta_tx::<3>(t, ld)?, self.beta_tx::<3>(t, rd)?);
        if b3_ld != NULL_DART_ID && b3_rd != NULL_DART_ID {
            self.betas.one_link_core(t, b3_rd, b3_ld)?;
        }
        Ok(())
    }
}

/// 1-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 1-unlink operation.
    pub(crate) fn one_unlink_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<DartIdType, LinkError> {
        let rd = self.beta_tx::<1>(t, ld)?;
        self.betas.one_unlink_core(t, ld)?;
        let (b3_ld, b3_rd) = (self.beta_tx::<3>(t, ld)?, self.beta_tx::<3>(t, rd)?);
        if b3_ld != NULL_DART_ID && b3_rd != NULL_DART_ID {
            if self.beta_tx::<1>(t, b3_rd)? != b3_ld {
                // FIXME: add dedicated variant ~LinkError::DivergentStructures ?
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            self.betas.one_unlink_core(t, b3_rd)?;
        }
        Ok(rd)
    }
}
