use crate::attributes::UnknownAttributeStorage;
use crate::cmap::{CMap2, DartIdType, NULL_DART_ID, OrbitPolicy, SewError};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, try_or_coerce};

#[doc(hidden)]
/// **1-(un)sews internals**
impl<T: CoordsFloat> CMap2<T> {
    /// 1-sew transactional implementation.
    pub(super) fn one_sew(
        &self,
        t: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b2lhs_dart_id = self.beta_tx::<2>(t, lhs_dart_id)?;
        if b2lhs_dart_id == NULL_DART_ID {
            try_or_coerce!(
                self.betas.one_link_core(t, lhs_dart_id, rhs_dart_id),
                SewError
            );
        } else {
            let b2lhs_vid_old = self.vertex_id_tx(t, b2lhs_dart_id)?;
            let rhs_vid_old = self.vertex_id_tx(t, rhs_dart_id)?;

            try_or_coerce!(
                self.betas.one_link_core(t, lhs_dart_id, rhs_dart_id),
                SewError
            );

            try_or_coerce!(
                self.vertices.merge(
                    t,
                    b2lhs_vid_old.min(rhs_vid_old),
                    b2lhs_vid_old,
                    rhs_vid_old,
                ),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    b2lhs_vid_old.min(rhs_vid_old),
                    b2lhs_vid_old,
                    rhs_vid_old,
                ),
                SewError
            );
        }
        Ok(())
    }

    /// 1-unsew transactional implementation.
    pub(super) fn one_unsew(
        &self,
        t: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b2lhs_dart_id = self.beta_tx::<2>(t, lhs_dart_id)?;
        if b2lhs_dart_id == NULL_DART_ID {
            try_or_coerce!(self.betas.one_unlink_core(t, lhs_dart_id), SewError);
        } else {
            // fetch IDs before topology update
            let rhs_dart_id = self.beta_tx::<1>(t, lhs_dart_id)?;
            // update the topology
            try_or_coerce!(self.betas.one_unlink_core(t, lhs_dart_id), SewError);
            // split vertices & attributes from the old ID to the new ones
            let (new_lhs, new_rhs) = (
                self.vertex_id_tx(t, b2lhs_dart_id)?,
                self.vertex_id_tx(t, rhs_dart_id)?,
            );
            try_or_coerce!(
                self.vertices
                    .split(t, new_lhs, new_rhs, new_lhs.min(new_rhs)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    new_lhs,
                    new_rhs,
                    new_lhs.min(new_rhs),
                ),
                SewError
            );
        }
        Ok(())
    }
}
