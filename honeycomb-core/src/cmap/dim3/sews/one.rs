//! 1D sew implementations

use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{CMap3, DartIdType, SewError, NULL_DART_ID, NULL_VERTEX_ID},
    geometry::CoordsFloat,
    stm::{try_or_coerce, Transaction, TransactionClosureResult},
};

#[doc(hidden)]
/// **1-(un)sews internals)**
impl<T: CoordsFloat> CMap3<T> {
    /// 1-sew transactional operation.
    pub(crate) fn one_sew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        // the main difference with 2D implementation is the beta 3 image check
        // if both darts have a b3 image, then we need to 1-link b3(rd) to b3(ld) as well
        // this is handled by `one_link`, but we need to merge old vertex data
        let b3ld = self.beta_transac::<3>(trans, ld)?;
        let b2ld = self.beta_transac::<2>(trans, ld)?;
        let vid_l_old = if b3ld != NULL_DART_ID {
            self.vertex_id_transac(trans, b3ld)?
        } else if b2ld != NULL_DART_ID {
            self.vertex_id_transac(trans, b2ld)?
        } else {
            NULL_VERTEX_ID
        };
        let vid_r_old = self.vertex_id_transac(trans, rd)?;

        try_or_coerce!(self.one_link(trans, ld, rd), SewError);

        if vid_l_old != NULL_VERTEX_ID {
            let new_vid = vid_r_old.min(vid_l_old); // is this correct?
            try_or_coerce!(
                self.vertices.merge(trans, new_vid, vid_l_old, vid_r_old),
                SewError
            );
            try_or_coerce!(
                self.attributes
                    .merge_vertex_attributes(trans, new_vid, vid_l_old, vid_r_old),
                SewError
            );
        }
        Ok(())
    }

    /// 1-unsew transactional operation.
    pub(crate) fn one_unsew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_transac::<1>(trans, ld)?;
        let vid_old = self.vertex_id_transac(trans, rd)?;

        try_or_coerce!(self.one_unlink(trans, ld), SewError);
        let b2ld = self.beta_transac::<2>(trans, ld)?;
        let b3ld = self.beta_transac::<3>(trans, ld)?;

        let vid_l_new = self.vertex_id_transac(
            trans,
            if b2ld != NULL_DART_ID {
                b2ld
            } else if b3ld != NULL_DART_ID {
                b3ld
            } else {
                // don't split if there's no vertex on one side
                return Ok(());
            },
        )?;
        let vid_r_new = self.vertex_id_transac(trans, rd)?;
        // perf: branch miss vs redundancy
        if vid_l_new != vid_r_new {
            try_or_coerce!(
                self.vertices.split(trans, vid_l_new, vid_r_new, vid_old),
                SewError
            );
            try_or_coerce!(
                self.attributes
                    .split_vertex_attributes(trans, vid_l_new, vid_r_new, vid_old),
                SewError
            );
        }
        Ok(())
    }
}
