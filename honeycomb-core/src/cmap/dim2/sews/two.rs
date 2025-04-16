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
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b1lhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
        let b1rhs_dart_id = self.betas[(1, rhs_dart_id)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => {
                try_or_coerce!(
                    self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id),
                    SewError
                );
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        eid_new,
                        lhs_dart_id as EdgeIdType, // valid in 2D
                        rhs_dart_id as EdgeIdType,
                    ),
                    SewError
                );
            }
            // update vertex associated to b1rhs/lhs
            (true, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = lhs_dart_id as EdgeIdType; // valid in 2D
                let rhs_eid_old = rhs_dart_id as EdgeIdType;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let b1rhs_vid_old = self.vertex_id_transac(trans, b1rhs_dart_id)?;
                // update the topology
                try_or_coerce!(
                    self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id),
                    SewError
                );
                // merge vertices & attributes from the old IDs to the new one
                let lhs_vid_new = self.vertex_id_transac(trans, lhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                try_or_coerce!(
                    self.vertices
                        .merge(trans, lhs_vid_new, lhs_vid_old, b1rhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        lhs_vid_new,
                        lhs_vid_old,
                        b1rhs_vid_old,
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        eid_new,
                        lhs_eid_old,
                        rhs_eid_old,
                    ),
                    SewError
                );
            }
            // update vertex associated to b1lhs/rhs
            (false, true) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = lhs_dart_id as EdgeIdType; // valid in 2D
                let rhs_eid_old = rhs_dart_id as EdgeIdType;
                let b1lhs_vid_old = self.vertex_id_transac(trans, b1lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                try_or_coerce!(
                    self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id),
                    SewError
                );
                // merge vertices & attributes from the old IDs to the new one
                let rhs_vid_new = self.vertex_id_transac(trans, rhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                try_or_coerce!(
                    self.vertices
                        .merge(trans, rhs_vid_new, b1lhs_vid_old, rhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        rhs_vid_new,
                        b1lhs_vid_old,
                        rhs_vid_old,
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        eid_new,
                        lhs_eid_old,
                        rhs_eid_old,
                    ),
                    SewError
                );
            }
            // update both vertices making up the edge
            (false, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = lhs_dart_id as EdgeIdType; // valid in 2D
                let rhs_eid_old = rhs_dart_id as EdgeIdType;
                // (lhs/b1rhs) vertex
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let b1rhs_vid_old = self.vertex_id_transac(trans, b1rhs_dart_id)?;
                // (b1lhs/rhs) vertex
                let b1lhs_vid_old = self.vertex_id_transac(trans, b1lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;

                // check orientation
                if let (
                    Ok(Some(l_vertex)),   // lhs
                    Ok(Some(b1r_vertex)), // b1rhs
                    Ok(Some(b1l_vertex)), // b1lhs
                    Ok(Some(r_vertex)),   // rhs
                ) = (
                    self.vertices.read(trans, lhs_vid_old),   // lhs
                    self.vertices.read(trans, b1rhs_vid_old), // b1rhs
                    self.vertices.read(trans, b1lhs_vid_old), // b1lhs
                    self.vertices.read(trans, rhs_vid_old),   // rhs
                ) {
                    let lhs_vector = b1l_vertex - l_vertex;
                    let rhs_vector = b1r_vertex - r_vertex;
                    // dot product should be negative if the two darts have opposite direction
                    // we could also put restriction on the angle made by the two darts to prevent
                    // drastic deformation
                    if lhs_vector.dot(&rhs_vector) >= T::zero() {
                        abort(SewError::BadGeometry(2, lhs_dart_id, rhs_dart_id))?;
                    }
                }

                // update the topology
                try_or_coerce!(
                    self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id),
                    SewError
                );
                // merge vertices & attributes from the old IDs to the new one
                let lhs_vid_new = self.vertex_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_new = self.vertex_id_transac(trans, rhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                try_or_coerce!(
                    self.vertices
                        .merge(trans, lhs_vid_new, lhs_vid_old, b1rhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.vertices
                        .merge(trans, rhs_vid_new, b1lhs_vid_old, rhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        lhs_vid_new,
                        lhs_vid_old,
                        b1rhs_vid_old,
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        rhs_vid_new,
                        b1lhs_vid_old,
                        rhs_vid_old,
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.merge_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        eid_new,
                        lhs_eid_old,
                        rhs_eid_old,
                    ),
                    SewError
                );
            }
        }
        Ok(())
    }

    /// 2-unsew transactional implementation.
    #[allow(clippy::too_many_lines)]
    pub(super) fn two_unsew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        let b1lhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
        let b1rhs_dart_id = self.betas[(1, rhs_dart_id)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            (true, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(trans, lhs_dart_id), SewError);
                // split attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        lhs_dart_id as EdgeIdType, // valid in 2D
                        rhs_dart_id as EdgeIdType,
                        eid_old,
                    ),
                    SewError
                );
            }
            (true, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(trans, lhs_dart_id), SewError);
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        lhs_dart_id as EdgeIdType, // valid in 2D
                        rhs_dart_id as EdgeIdType,
                        eid_old,
                    ),
                    SewError
                );
                let (new_lv_lhs, new_lv_rhs) = (
                    self.vertex_id_transac(trans, lhs_dart_id)?,
                    self.vertex_id_transac(trans, b1rhs_dart_id)?,
                );
                try_or_coerce!(
                    self.vertices
                        .split(trans, new_lv_lhs, new_lv_rhs, lhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        new_lv_lhs,
                        new_lv_rhs,
                        lhs_vid_old,
                    ),
                    SewError
                );
            }
            (false, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(trans, lhs_dart_id), SewError);
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        lhs_dart_id as EdgeIdType, // valid in 2D
                        rhs_dart_id as EdgeIdType,
                        eid_old,
                    ),
                    SewError
                );
                let (new_rv_lhs, new_rv_rhs) = (
                    self.vertex_id_transac(trans, b1lhs_dart_id)?,
                    self.vertex_id_transac(trans, rhs_dart_id)?,
                );
                try_or_coerce!(
                    self.vertices
                        .split(trans, new_rv_lhs, new_rv_rhs, rhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        new_rv_lhs,
                        new_rv_rhs,
                        rhs_vid_old,
                    ),
                    SewError
                );
            }
            (false, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                try_or_coerce!(self.betas.two_unlink_core(trans, lhs_dart_id), SewError);
                // split vertices & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Edge,
                        lhs_dart_id as EdgeIdType, // valid in 2D
                        rhs_dart_id as EdgeIdType,
                        eid_old,
                    ),
                    SewError
                );
                let (new_lv_lhs, new_lv_rhs) = (
                    self.vertex_id_transac(trans, lhs_dart_id)?,
                    self.vertex_id_transac(trans, b1rhs_dart_id)?,
                );
                let (new_rv_lhs, new_rv_rhs) = (
                    self.vertex_id_transac(trans, b1lhs_dart_id)?,
                    self.vertex_id_transac(trans, rhs_dart_id)?,
                );
                try_or_coerce!(
                    self.vertices
                        .split(trans, new_lv_lhs, new_lv_rhs, lhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        new_lv_lhs,
                        new_lv_rhs,
                        lhs_vid_old,
                    ),
                    SewError
                );
                try_or_coerce!(
                    self.vertices
                        .split(trans, new_rv_lhs, new_rv_rhs, rhs_vid_old),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        trans,
                        OrbitPolicy::Vertex,
                        new_rv_lhs,
                        new_rv_rhs,
                        rhs_vid_old,
                    ),
                    SewError
                );
            }
        }
        Ok(())
    }
}
