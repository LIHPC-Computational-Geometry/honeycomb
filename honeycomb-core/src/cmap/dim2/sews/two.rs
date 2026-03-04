use crate::attributes::{AttributeStorage, UnknownAttributeStorage};
use crate::cmap::{CMap2, DartIdType, EdgeIdType, NULL_DART_ID, OrbitPolicy, SewError};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, abort, try_or_coerce};

#[doc(hidden)]
/// **2-(un)sews internals**
impl<T: CoordsFloat> CMap2<T> {
    /// 2-sew transactional implementation.
    #[allow(clippy::too_many_lines)]
    pub(super) fn two_sew(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b1ld = self.beta_tx::<1>(t, ld)?;
        let b1rd = self.beta_tx::<1>(t, rd)?;

        // fetch vertices ID before topology update
        let ld_eid = ld as EdgeIdType; // valid in 2D
        let rd_eid = rd as EdgeIdType;
        // (ld/b1rd) vertex
        let ld_vid = self.vertex_id_tx(t, ld)?;
        let b1rd_vid = self.vertex_id_tx(t, b1rd)?;
        // (b1ld/rd) vertex
        let b1ld_vid = self.vertex_id_tx(t, b1ld)?;
        let rd_vid = self.vertex_id_tx(t, rd)?;

        // check orientation
        if let (
            Some(l_vertex),   // ld
            Some(b1r_vertex), // b1rd
            Some(b1l_vertex), // b1ld
            Some(r_vertex),   // rd
        ) = (
            self.vertices.read(t, ld_vid)?,   // ld
            self.vertices.read(t, b1rd_vid)?, // b1rd
            self.vertices.read(t, b1ld_vid)?, // b1ld
            self.vertices.read(t, rd_vid)?,   // rd
        ) {
            let ld_vector = b1l_vertex - l_vertex;
            let rd_vector = b1r_vertex - r_vertex;
            // dot product should be negative if the two darts have opposite direction
            // we could also put restriction on the angle made by the two darts to prevent
            // drastic deformation
            if ld_vector.dot(&rd_vector) >= T::zero() {
                abort(SewError::BadGeometry(2, ld, rd))?;
            }
        }

        try_or_coerce!(self.link_tx::<2>(t, ld, rd), SewError);

        // merge edge attributes
        try_or_coerce!(
            self.attributes.merge_attributes(
                t,
                OrbitPolicy::Edge,
                ld_eid.min(rd_eid),
                ld_eid,
                rd_eid,
            ),
            SewError
        );

        // merge vertices & attributes from the old IDs to the new one
        if b1rd != NULL_DART_ID {
            try_or_coerce!(
                self.vertices
                    .merge(t, ld_vid.min(b1rd_vid), ld_vid, b1rd_vid),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    ld_vid.min(b1rd_vid),
                    ld_vid,
                    b1rd_vid,
                ),
                SewError
            );
        }
        if b1ld != NULL_DART_ID {
            try_or_coerce!(
                self.vertices
                    .merge(t, b1ld_vid.min(rd_vid), b1ld_vid, rd_vid),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    b1ld_vid.min(rd_vid),
                    b1ld_vid,
                    rd_vid,
                ),
                SewError
            );
        }

        Ok(())
    }

    /// 2-unsew transactional implementation.
    #[allow(clippy::too_many_lines)]
    pub(super) fn two_unsew(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_tx::<2>(t, ld)?;
        let b1ld = self.beta_tx::<1>(t, ld)?;
        let b1rd = self.beta_tx::<1>(t, rd)?;

        try_or_coerce!(self.unlink_tx::<2>(t, ld), SewError);

        let (new_lv_ld, new_lv_rd) = (self.vertex_id_tx(t, ld)?, self.vertex_id_tx(t, b1rd)?);
        let (new_rv_ld, new_rv_rd) = (self.vertex_id_tx(t, b1ld)?, self.vertex_id_tx(t, rd)?);

        // split edge attributes
        try_or_coerce!(
            self.attributes.split_attributes(
                t,
                OrbitPolicy::Edge,
                ld as EdgeIdType, // valid in 2D
                rd as EdgeIdType,
                ld.min(rd) as EdgeIdType,
            ),
            SewError
        );
        // split vertex attributes
        if b1rd != NULL_DART_ID {
            try_or_coerce!(
                self.vertices
                    .split(t, new_lv_ld, new_lv_rd, new_lv_ld.min(new_lv_rd)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    new_lv_ld,
                    new_lv_rd,
                    new_lv_ld.min(new_lv_rd),
                ),
                SewError
            );
        }
        if b1ld != NULL_DART_ID {
            try_or_coerce!(
                self.vertices
                    .split(t, new_rv_ld, new_rv_rd, new_rv_ld.min(new_rv_rd)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    new_rv_ld,
                    new_rv_rd,
                    new_rv_ld.min(new_rv_rd),
                ),
                SewError
            );
        }

        Ok(())
    }
}
