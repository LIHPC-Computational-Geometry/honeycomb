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
        let b1ld = self.betas[(1, ld)].read(t)?;
        let b1rd = self.betas[(1, rd)].read(t)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1ld == NULL_DART_ID, b1rd == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => {
                let eid_l = self.edge_id_tx(t, ld)?;
                let eid_r = self.edge_id_tx(t, rd)?;
                try_or_coerce!(self.betas.two_link_core(t, ld, rd), SewError);
                let eid_new = self.edge_id_tx(t, ld)?;
                try_or_coerce!(
                    self.attributes
                        .merge_attributes(t, OrbitPolicy::Edge, eid_new, eid_l, eid_r),
                    SewError
                );
            }
            // update vertex associated to b1rhs/lhs
            (true, false) => {
                // fetch vertices ID before topology update
                let eid_l = self.edge_id_tx(t, ld)?;
                let eid_r = self.edge_id_tx(t, rd)?;
                let vid_l = self.vertex_id_tx(t, ld)?;
                let vid_b1r = self.vertex_id_tx(t, b1rd)?;
                // update the topology
                try_or_coerce!(self.betas.two_link_core(t, ld, rd), SewError);
                // merge vertices & attributes from the old IDs to the new one
                let vid_l_new = self.vertex_id_tx(t, ld)?;
                let eid_new = self.edge_id_tx(t, ld)?;
                try_or_coerce!(self.vertices.merge(t, vid_l_new, vid_l, vid_b1r), SewError);
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_l_new,
                        vid_l,
                        vid_b1r
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes
                        .merge_attributes(t, OrbitPolicy::Edge, eid_new, eid_l, eid_r),
                    SewError
                );
            }
            // update vertex associated to b1lhs/rhs
            (false, true) => {
                // fetch vertices ID before topology update
                let eid_l = self.edge_id_tx(t, ld)?;
                let eid_r = self.edge_id_tx(t, rd)?;
                let vid_b1l = self.vertex_id_tx(t, b1ld)?;
                let vid_r = self.vertex_id_tx(t, rd)?;
                // update the topology
                try_or_coerce!(self.betas.two_link_core(t, ld, rd), SewError);
                // merge vertices & attributes from the old IDs to the new one
                let vid_r_new = self.vertex_id_tx(t, rd)?;
                let eid_new = self.edge_id_tx(t, ld)?;
                try_or_coerce!(self.vertices.merge(t, vid_r_new, vid_b1l, vid_r), SewError);
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_r_new,
                        vid_b1l,
                        vid_r
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes
                        .merge_attributes(t, OrbitPolicy::Edge, eid_new, eid_l, eid_r),
                    SewError
                );
            }
            // update both vertices making up the edge
            (false, false) => {
                // fetch vertices ID before topology update
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

                // update the topology
                try_or_coerce!(self.betas.two_link_core(t, ld, rd), SewError);
                // merge vertices & attributes from the old IDs to the new one
                let vid_l_new = self.vertex_id_tx(t, ld)?;
                let vid_r_new = self.vertex_id_tx(t, rd)?;
                let eid_new = self.edge_id_tx(t, ld)?;
                try_or_coerce!(self.vertices.merge(t, vid_l_new, vid_l, vid_b1r), SewError);
                try_or_coerce!(self.vertices.merge(t, vid_r_new, vid_b1l, vid_r), SewError);
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_l_new,
                        vid_l,
                        vid_b1r
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_r_new,
                        vid_b1l,
                        vid_r
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes
                        .merge_attributes(t, OrbitPolicy::Edge, eid_new, eid_l, eid_r),
                    SewError
                );
            }
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
        let rd = self.betas[(2, ld)].read(t)?;
        let b1ld = self.betas[(1, ld)].read(t)?;
        let b1rd = self.betas[(1, rd)].read(t)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1ld == NULL_DART_ID, b1rd == NULL_DART_ID) {
            (true, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_tx(t, ld)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(t, ld), SewError);
                // split attributes from the old ID to the new ones
                let (eid_newl, eid_newr) = (self.edge_id_tx(t, ld)?, self.edge_id_tx(t, rd)?);
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Edge,
                        eid_newl,
                        eid_newr,
                        eid_old
                    ),
                    SewError
                );
            }
            (true, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_tx(t, ld)?;
                let vid_l = self.vertex_id_tx(t, ld)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(t, ld), SewError);
                // split vertex & attributes from the old ID to the new ones
                let (eid_newl, eid_newr) = (self.edge_id_tx(t, ld)?, self.edge_id_tx(t, rd)?);
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Edge,
                        eid_newl,
                        eid_newr,
                        eid_old
                    ),
                    SewError
                );
                let (vid_l_newl, vid_l_newr) =
                    (self.vertex_id_tx(t, ld)?, self.vertex_id_tx(t, b1rd)?);
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                try_or_coerce!(
                    self.vertices.split(t, vid_l_newl, vid_l_newr, vid_l),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_l_newl,
                        vid_l_newr,
                        vid_l
                    ),
                    SewError
                );
            }
            (false, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_tx(t, ld)?;
                let vid_r = self.vertex_id_tx(t, rd)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(t, ld), SewError);
                // split vertex & attributes from the old ID to the new ones
                let (eid_newl, eid_newr) = (self.edge_id_tx(t, ld)?, self.edge_id_tx(t, rd)?);
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Edge,
                        eid_newl,
                        eid_newr,
                        eid_old
                    ),
                    SewError
                );
                let (vid_r_newl, vid_r_newr) =
                    (self.vertex_id_tx(t, b1ld)?, self.vertex_id_tx(t, rd)?);
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                try_or_coerce!(
                    self.vertices.split(t, vid_r_newl, vid_r_newr, vid_r),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_r_newl,
                        vid_r_newr,
                        vid_r
                    ),
                    SewError
                );
            }
            (false, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_tx(t, ld)?;
                let vid_l = self.vertex_id_tx(t, ld)?;
                let vid_r = self.vertex_id_tx(t, rd)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(t, ld), SewError);
                // split vertices & attributes from the old ID to the new ones
                let (eid_newl, eid_newr) = (self.edge_id_tx(t, ld)?, self.edge_id_tx(t, rd)?);
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Edge,
                        eid_newl,
                        eid_newr,
                        eid_old
                    ),
                    SewError
                );
                let (vid_l_newl, vid_l_newr) =
                    (self.vertex_id_tx(t, ld)?, self.vertex_id_tx(t, b1rd)?);
                let (vid_r_newl, vid_r_newr) =
                    (self.vertex_id_tx(t, b1ld)?, self.vertex_id_tx(t, rd)?);
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                try_or_coerce!(
                    self.vertices.split(t, vid_l_newl, vid_l_newr, vid_l),
                    SewError
                );
                try_or_coerce!(
                    self.vertices.split(t, vid_r_newl, vid_r_newr, vid_r),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_l_newl,
                        vid_l_newr,
                        vid_l
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_r_newl,
                        vid_r_newr,
                        vid_r
                    ),
                    SewError
                );
            }
        }
        Ok(())
    }
}
