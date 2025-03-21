//! 1D link implementations

use crate::cmap::{CMap3, DartIdType, LinkError, NULL_DART_ID};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, abort, atomically_with_err};

/// 1-links
impl<T: CoordsFloat> CMap3<T> {
    /// 1-link operation.
    pub(crate) fn one_link(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
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
    pub(crate) fn force_one_link(&self, ld: DartIdType, rd: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.one_link(trans, ld, rd))
    }
}

/// 1-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 1-unlink operation.
    pub(crate) fn one_unlink(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        let rd = self.beta_transac::<1>(trans, ld)?;
        self.betas.one_unlink_core(trans, ld)?;
        let (b3_ld, b3_rd) = (
            self.beta_transac::<3>(trans, ld)?,
            self.beta_transac::<3>(trans, rd)?,
        );
        if b3_ld != NULL_DART_ID && b3_rd != NULL_DART_ID {
            if self.beta_transac::<1>(trans, b3_rd)? != b3_ld {
                // FIXME: add dedicated variant ~LinkError::DivergentStructures ?
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            self.betas.one_unlink_core(trans, b3_rd)?;
        }
        Ok(())
    }

    /// 1-unlink operation.
    ///
    /// This variant is equivalent to `one_unlink`, but internally uses a transaction that will be
    /// retried until validated.
    pub(crate) fn force_one_unlink(&self, ld: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|trans| self.one_unlink(trans, ld))
    }
}
