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
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b2ld = self.beta_tx::<2>(t, ld)?;
        let b2l_vid = self.vertex_id_tx(t, b2ld)?;
        let r_vid = self.vertex_id_tx(t, rd)?;

        try_or_coerce!(self.link::<1>(t, ld, rd), SewError);

        if b2ld != NULL_DART_ID {
            try_or_coerce!(
                self.vertices.merge(t, b2l_vid.min(r_vid), b2l_vid, r_vid,),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    b2l_vid.min(r_vid),
                    b2l_vid,
                    r_vid,
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
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_tx::<1>(t, ld)?;
        let b2ld = self.beta_tx::<2>(t, ld)?;

        try_or_coerce!(self.unlink::<1>(t, ld), SewError);

        if b2ld != NULL_DART_ID {
            // split vertex attributes
            let (new_lhs, new_rhs) = (self.vertex_id_tx(t, b2ld)?, self.vertex_id_tx(t, rd)?);
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
