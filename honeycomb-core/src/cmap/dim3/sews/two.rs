//! 2D sew implementations

use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    cmap::{CMap3, DartIdType, NULL_DART_ID, OrbitPolicy, SewError},
    geometry::CoordsFloat,
    stm::{Transaction, TransactionClosureResult, abort, try_or_coerce},
};

/// **2-(un)sews internals**
impl<T: CoordsFloat> CMap3<T> {
    #[allow(clippy::too_many_lines)]
    /// 2-sew transactional operation.
    pub(crate) fn two_sew(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b1ld = self.beta_tx::<1>(t, ld)?.max(self.beta_tx::<3>(t, ld)?);
        let b1rd = self.beta_tx::<1>(t, rd)?.max(self.beta_tx::<3>(t, rd)?);

        // fetch cell IDs before link
        let eid_l = self.edge_id_tx(t, ld)?;
        let eid_r = self.edge_id_tx(t, rd)?;
        // (lhs/b1rhs) vertex
        let vid_l = self.vertex_id_tx(t, ld)?;
        let vid_b1r = self.vertex_id_tx(t, b1rd)?;
        // (b1lhs/rhs) vertex
        let vid_b1l = self.vertex_id_tx(t, b1ld)?;
        let vid_r = self.vertex_id_tx(t, rd)?;

        // check orientation
        if let (
            // (lhs/b1rhs) vertices
            Ok(Some(l_vertex)),
            Ok(Some(b1r_vertex)),
            // (b1lhs/rhs) vertices
            Ok(Some(b1l_vertex)),
            Ok(Some(r_vertex)),
        ) = (
            // (lhs/b1rhs)
            self.vertices.read(t, vid_l),
            self.vertices.read(t, vid_b1r),
            // (b1lhs/rhs)
            self.vertices.read(t, vid_b1l),
            self.vertices.read(t, vid_r),
        ) {
            let lhs_vector = b1l_vertex - l_vertex;
            let rhs_vector = b1r_vertex - r_vertex;
            // dot product should be negative if the two darts have opposite direction
            // we could also put restriction on the angle made by the two darts to prevent
            // drastic deformation
            if lhs_vector.dot(&rhs_vector) >= T::zero() {
                abort(SewError::BadGeometry(2, ld, rd))?;
            }
        }

        try_or_coerce!(self.betas.two_link_core(t, ld, rd), SewError);

        // merge edge attributes
        try_or_coerce!(
            self.attributes
                .merge_attributes(t, OrbitPolicy::Edge, eid_l.min(eid_r), eid_l, eid_r),
            SewError
        );

        // merge vertex attributes depending on whether
        // - there was an existing orbit at each end
        // - there was an existing orbit on each side
        if b1rd != NULL_DART_ID {
            try_or_coerce!(
                self.vertices.merge(t, vid_l.min(vid_b1r), vid_l, vid_b1r),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_l.min(vid_b1r),
                    vid_l,
                    vid_b1r
                ),
                SewError
            );
        }
        if b1ld != NULL_DART_ID {
            try_or_coerce!(
                self.vertices.merge(t, vid_b1l.min(vid_r), vid_b1l, vid_r),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_b1l.min(vid_r),
                    vid_b1l,
                    vid_r
                ),
                SewError
            );
        }

        Ok(())
    }

    /// 2-unsew transactional operation.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn two_unsew(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_tx::<2>(t, ld)?;
        let b1ld = self.beta_tx::<1>(t, ld)?.max(self.beta_tx::<3>(t, ld)?);
        let b1rd = self.beta_tx::<1>(t, rd)?.max(self.beta_tx::<3>(t, rd)?);

        try_or_coerce!(self.betas.two_unlink_core(t, ld), SewError);

        let (eid_newl, eid_newr) = (self.edge_id_tx(t, ld)?, self.edge_id_tx(t, rd)?);
        let (vid_l_newl, vid_l_newr) = (self.vertex_id_tx(t, ld)?, self.vertex_id_tx(t, b1rd)?);
        let (vid_r_newl, vid_r_newr) = (self.vertex_id_tx(t, b1ld)?, self.vertex_id_tx(t, rd)?);

        // split edge attributes
        try_or_coerce!(
            self.attributes.split_attributes(
                t,
                OrbitPolicy::Edge,
                eid_newl,
                eid_newr,
                eid_newl.min(eid_newr),
            ),
            SewError
        );

        // split vertex attributes depending on whether two distinct orbits were formed by unlink
        if b1rd != NULL_DART_ID && vid_l_newl != vid_r_newr {
            try_or_coerce!(
                self.vertices
                    .split(t, vid_l_newl, vid_l_newr, vid_l_newl.min(vid_l_newr)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_l_newl,
                    vid_l_newr,
                    vid_l_newl.min(vid_l_newr)
                ),
                SewError
            );
        }
        if b1ld != NULL_DART_ID && vid_r_newl != vid_r_newr {
            try_or_coerce!(
                self.vertices
                    .split(t, vid_r_newl, vid_r_newr, vid_r_newl.min(vid_r_newr)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_r_newl,
                    vid_r_newr,
                    vid_r_newl.min(vid_r_newr)
                ),
                SewError
            );
        }

        Ok(())
    }
}
