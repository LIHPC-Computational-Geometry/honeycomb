//! 3D link implementations

use crate::cmap::{CMap3, DartIdType, LinkError, NULL_DART_ID};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, abort, atomically_with_err};

/// 3-links
impl<T: CoordsFloat> CMap3<T> {
    /// 3-link operation.
    pub(crate) fn three_link_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.three_link_core(t, ld, rd)?;
        let (mut lside, mut rside) = (self.beta_tx::<1>(t, ld)?, self.beta_tx::<0>(t, rd)?);
        // while we haven't completed the loop, or reached an end
        while lside != ld && lside != NULL_DART_ID {
            if rside == NULL_DART_ID {
                // (*)
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            self.betas.three_link_core(t, lside, rside)?;
            (lside, rside) = (self.beta_tx::<1>(t, lside)?, self.beta_tx::<0>(t, rside)?);
        }
        // the face was open, so we need to cover the other direction
        // for meshes, we should be working on complete faces at all times,
        // so branch prediction will hopefully save use
        if lside == NULL_DART_ID {
            if rside != NULL_DART_ID {
                // (*)
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            (lside, rside) = (self.beta_tx::<0>(t, ld)?, self.beta_tx::<1>(t, rd)?);
            while lside != NULL_DART_ID {
                if rside == NULL_DART_ID {
                    // (*)
                    abort(LinkError::AsymmetricalFaces(ld, rd))?;
                }
                self.betas.three_link_core(t, lside, rside)?;
                (lside, rside) = (self.beta_tx::<0>(t, lside)?, self.beta_tx::<1>(t, rside)?);
            }
        }
        // (*): if we land on NULL on one side, the other side should be NULL as well
        //      if that is not the case, it means (either):
        //      - we're trying to sew open faces with a different number of darts
        //      - we're trying to sew open faces that are offset by one (or more) dart(s)
        //      in both case, this is way too clunky to be considered valid
        Ok(())
    }

    /// 3-link operation.
    pub(crate) fn three_link(&self, ld: DartIdType, rd: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|t| self.three_link_tx(t, ld, rd))
    }
}

/// 3-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 3-unlink operation.
    pub(crate) fn three_unlink_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        let rd = self.beta_tx::<3>(t, ld)?;

        self.betas.three_unlink_core(t, ld)?;
        let (mut lside, mut rside) = (self.beta_tx::<1>(t, ld)?, self.beta_tx::<0>(t, rd)?);
        // while we haven't completed the loop, or reached an end
        while lside != ld && lside != NULL_DART_ID {
            if lside != self.beta_tx::<3>(t, rside)? {
                // (*); FIXME: add dedicated err ~LinkError::DivergentStructures ?
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            self.betas.three_unlink_core(t, lside)?;
            (lside, rside) = (self.beta_tx::<1>(t, lside)?, self.beta_tx::<0>(t, rside)?);
        }
        // the face was open, so we need to cover the other direction
        // for meshes, we should be working on complete faces at all times,
        // so branch prediction will hopefully save use
        if lside == NULL_DART_ID {
            if rside != NULL_DART_ID {
                // (**)
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            (lside, rside) = (self.beta_tx::<0>(t, ld)?, self.beta_tx::<1>(t, rd)?);
            while lside != NULL_DART_ID {
                if lside != self.beta_tx::<3>(t, rside)? {
                    // (*); FIXME: add dedicated err ~LinkError::DivergentStructures ?
                    abort(LinkError::AsymmetricalFaces(ld, rd))?;
                }
                assert_eq!(lside, self.beta_tx::<3>(t, rside)?); // (*)
                self.betas.three_unlink_core(t, lside)?;
                (lside, rside) = (self.beta_tx::<0>(t, lside)?, self.beta_tx::<1>(t, rside)?);
            }
        }
        // (*) : this can be changed, but the idea here is to ensure we're unlinking the expected
        //       construct
        // (**): if we land on NULL on one side, the other side should be NULL as well
        Ok(())
    }

    /// 3-unlink operation.
    pub(crate) fn three_unlink(&self, ld: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|t| self.three_unlink_tx(t, ld))
    }
}
