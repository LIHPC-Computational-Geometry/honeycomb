//! 1D sew implementations

use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{CMap3, DartIdType, NULL_DART_ID, OrbitPolicy, SewError},
    geometry::CoordsFloat,
    stm::{Transaction, TransactionClosureResult, try_or_coerce},
};

#[doc(hidden)]
/// **1-(un)sews internals)**
impl<T: CoordsFloat> CMap3<T> {
    /// 1-sew transactional operation.
    pub(crate) fn one_sew_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        // the main difference with 2D implementation is the beta 3 image check
        // if both darts have a b3 image, then we need to 1-link b3(rd) to b3(ld) as well
        // this is handled by `one_link`, but we need to merge old vertex data
        let b3ld = self.beta_tx::<3>(t, ld)?.max(self.beta_tx::<2>(t, ld)?);
        let vid_l_old = self.vertex_id_tx(t, b3ld)?;
        let vid_r_old = self.vertex_id_tx(t, rd)?;

        try_or_coerce!(self.one_link_tx(t, ld, rd), SewError);

        if b3ld != NULL_DART_ID {
            let new_vid = vid_r_old.min(vid_l_old);
            try_or_coerce!(
                self.vertices.merge(t, new_vid, vid_l_old, vid_r_old),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    new_vid,
                    vid_l_old,
                    vid_r_old
                ),
                SewError
            );
        }
        Ok(())
    }

    /// 1-unsew transactional operation.
    pub(crate) fn one_unsew_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_tx::<1>(t, ld)?;
        let b3ld = self.beta_tx::<3>(t, ld)?.max(self.beta_tx::<2>(t, ld)?);

        try_or_coerce!(self.one_unlink_tx(t, ld), SewError);

        let vid_l_new = self.vertex_id_tx(t, b3ld)?;
        let vid_r_new = self.vertex_id_tx(t, rd)?;
        // perf: branch miss vs redundancy
        if b3ld != NULL_DART_ID && vid_l_new != vid_r_new {
            try_or_coerce!(
                self.vertices
                    .split(t, vid_l_new, vid_r_new, vid_l_new.min(vid_r_new)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_l_new,
                    vid_r_new,
                    vid_l_new.min(vid_r_new),
                ),
                SewError
            );
        }
        Ok(())
    }
}
