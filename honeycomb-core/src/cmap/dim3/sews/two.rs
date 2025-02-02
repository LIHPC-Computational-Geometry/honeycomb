//! 2D sew implementations

use crate::stm::{atomically, Transaction};

use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    cmap::{CMap3, CMapResult, DartIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

/// 2-sews
impl<T: CoordsFloat> CMap3<T> {
    /// 2-sew transactional operation.
    pub(crate) fn two_sew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> CMapResult<()> {
        let b1ld = self.betas[(1, ld)].read(trans)?;
        let b1rd = self.betas[(1, rd)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1ld == NULL_DART_ID, b1rd == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => {
                self.betas.two_link_core(trans, ld, rd)?;
            }
            // update vertex associated to b1rhs/lhs
            (true, false) => {
                // fetch vertices ID before topology update
                let eid_l = self.edge_id_transac(trans, ld)?;
                let eid_r = self.edge_id_transac(trans, b1rd)?;
                let vid_l = self.vertex_id_transac(trans, ld)?;
                let vid_b1r = self.vertex_id_transac(trans, b1rd)?;
                // update the topology
                self.betas.two_link_core(trans, ld, rd)?;
                // merge vertices & attributes from the old IDs to the new one
                let vid_l_new = self.vertex_id_transac(trans, ld)?;
                let eid_new = self.edge_id_transac(trans, ld)?;
                self.vertices.try_merge(trans, vid_l_new, vid_l, vid_b1r)?;
                self.attributes
                    .try_merge_vertex_attributes(trans, vid_l_new, vid_l, vid_b1r)?;
                self.attributes
                    .try_merge_edge_attributes(trans, eid_new, eid_l, eid_r)?;
            }
            // update vertex associated to b1lhs/rhs
            (false, true) => {
                // fetch vertices ID before topology update
                let eid_l = self.edge_id_transac(trans, ld)?;
                let eid_r = self.edge_id_transac(trans, b1rd)?;
                let vid_b1l = self.vertex_id_transac(trans, b1ld)?;
                let vid_r = self.vertex_id_transac(trans, rd)?;
                // update the topology
                self.betas.two_link_core(trans, ld, rd)?;
                // merge vertices & attributes from the old IDs to the new one
                let vid_r_new = self.vertex_id_transac(trans, rd)?;
                let eid_new = self.edge_id_transac(trans, ld)?;
                self.vertices.try_merge(trans, vid_r_new, vid_b1l, vid_r)?;
                self.attributes
                    .try_merge_vertex_attributes(trans, vid_r_new, vid_b1l, vid_r)?;
                self.attributes
                    .try_merge_edge_attributes(trans, eid_new, eid_l, eid_r)?;
            }
            // update both vertices making up the edge
            (false, false) => {
                // fetch vertices ID before topology update
                let eid_l = self.edge_id_transac(trans, ld)?;
                let eid_r = self.edge_id_transac(trans, b1rd)?;
                // (lhs/b1rhs) vertex
                let vid_l = self.vertex_id_transac(trans, ld)?;
                let vid_b1r = self.vertex_id_transac(trans, b1rd)?;
                // (b1lhs/rhs) vertex
                let vid_b1l = self.vertex_id_transac(trans, b1ld)?;
                let vid_r = self.vertex_id_transac(trans, rd)?;

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
                    self.vertices.read(trans, vid_l),
                    self.vertices.read(trans, vid_b1r),
                    // (b1lhs/rhs)
                    self.vertices.read(trans, vid_b1l),
                    self.vertices.read(trans, vid_r),
                ) {
                    let lhs_vector = b1l_vertex - l_vertex;
                    let rhs_vector = b1r_vertex - r_vertex;
                    // dot product should be negative if the two darts have opposite direction
                    // we could also put restriction on the angle made by the two darts to prevent
                    // drastic deformation
                    assert!(
                        lhs_vector.dot(&rhs_vector) < T::zero(),
                        "{}",
                        format!(
                            "Dart {ld} and {rd} do not have consistent orientation for 2-sewing"
                        ),
                    );
                };

                // update the topology
                self.betas.two_link_core(trans, ld, rd)?;
                // merge vertices & attributes from the old IDs to the new one
                let vid_l_new = self.vertex_id_transac(trans, ld)?;
                let vid_r_new = self.vertex_id_transac(trans, rd)?;
                let eid_new = self.edge_id_transac(trans, ld)?;
                self.vertices.try_merge(trans, vid_l_new, vid_l, vid_b1r)?;
                self.vertices.try_merge(trans, vid_r_new, vid_b1l, vid_r)?;
                self.attributes
                    .try_merge_vertex_attributes(trans, vid_l_new, vid_l, vid_b1r)?;
                self.attributes
                    .try_merge_vertex_attributes(trans, vid_r_new, vid_b1l, vid_r)?;
                self.attributes
                    .try_merge_edge_attributes(trans, eid_new, eid_l, eid_r)?;
            }
        }
        Ok(())
    }

    /// 2-sew operation.
    pub(crate) fn force_two_sew(&self, ld: DartIdType, rd: DartIdType) {
        atomically(|trans| {
            let b1ld = self.betas[(1, ld)].read(trans)?;
            let b1rd = self.betas[(1, rd)].read(trans)?;
            // match (is lhs 1-free, is rhs 1-free)
            match (b1ld == NULL_DART_ID, b1rd == NULL_DART_ID) {
                // trivial case, no update needed
                (true, true) => {
                    self.betas.two_link_core(trans, ld, rd)?;
                }
                // update vertex associated to b1rhs/lhs
                (true, false) => {
                    // fetch vertices ID before topology update
                    let eid_l = self.edge_id_transac(trans, ld)?;
                    let eid_r = self.edge_id_transac(trans, b1rd)?;
                    let vid_l = self.vertex_id_transac(trans, ld)?;
                    let vid_b1r = self.vertex_id_transac(trans, b1rd)?;
                    // update the topology
                    self.betas.two_link_core(trans, ld, rd)?;
                    // merge vertices & attributes from the old IDs to the new one
                    let vid_l_new = self.vertex_id_transac(trans, ld)?;
                    let eid_new = self.edge_id_transac(trans, ld)?;
                    self.vertices.merge(trans, vid_l_new, vid_l, vid_b1r)?;
                    self.attributes
                        .merge_vertex_attributes(trans, vid_l_new, vid_l, vid_b1r)?;
                    self.attributes
                        .merge_edge_attributes(trans, eid_new, eid_l, eid_r)?;
                }
                // update vertex associated to b1lhs/rhs
                (false, true) => {
                    // fetch vertices ID before topology update
                    let eid_l = self.edge_id_transac(trans, ld)?;
                    let eid_r = self.edge_id_transac(trans, b1rd)?;
                    let vid_b1l = self.vertex_id_transac(trans, b1ld)?;
                    let vid_r = self.vertex_id_transac(trans, rd)?;
                    // update the topology
                    self.betas.two_link_core(trans, ld, rd)?;
                    // merge vertices & attributes from the old IDs to the new one
                    let vid_r_new = self.vertex_id_transac(trans, rd)?;
                    let eid_new = self.edge_id_transac(trans, ld)?;
                    self.vertices.merge(trans, vid_r_new, vid_b1l, vid_r)?;
                    self.attributes
                        .merge_vertex_attributes(trans, vid_r_new, vid_b1l, vid_r)?;
                    self.attributes
                        .merge_edge_attributes(trans, eid_new, eid_l, eid_r)?;
                }
                // update both vertices making up the edge
                (false, false) => {
                    // fetch vertices ID before topology update
                    let eid_l = self.edge_id_transac(trans, ld)?;
                    let eid_r = self.edge_id_transac(trans, b1rd)?;
                    // (lhs/b1rhs) vertex
                    let vid_l = self.vertex_id_transac(trans, ld)?;
                    let vid_b1r = self.vertex_id_transac(trans, b1rd)?;
                    // (b1lhs/rhs) vertex
                    let vid_b1l = self.vertex_id_transac(trans, b1ld)?;
                    let vid_r = self.vertex_id_transac(trans, rd)?;

                    // check orientation
                    if let (
                        // (lhs/b1rhs) vertices
                        Some(l_vertex),
                        Some(b1r_vertex),
                        // (b1lhs/rhs) vertices
                        Some(b1l_vertex),
                        Some(r_vertex),
                    ) = (
                        // (lhs/b1rhs)
                        self.vertices.read(trans, vid_l)?,
                        self.vertices.read(trans, vid_b1r)?,
                        // (b1lhs/rhs)
                        self.vertices.read(trans, vid_b1l)?,
                        self.vertices.read(trans, vid_r)?,
                    ) {
                        let lhs_vector = b1l_vertex - l_vertex;
                        let rhs_vector = b1r_vertex - r_vertex;
                        // dot product should be negative if the two darts have opposite direction
                        // we could also put restriction on the angle made by the two darts to prevent
                        // drastic deformation
                        assert!(
                            lhs_vector.dot(&rhs_vector) < T::zero(),
                            "{}",
                            format!(
                            "Dart {ld} and {rd} do not have consistent orientation for 2-sewing"
                        ),
                        );
                    };

                    // update the topology
                    self.betas.two_link_core(trans, ld, rd)?;
                    // merge vertices & attributes from the old IDs to the new one
                    let vid_l_new = self.vertex_id_transac(trans, ld)?;
                    let vid_r_new = self.vertex_id_transac(trans, rd)?;
                    let eid_new = self.edge_id_transac(trans, ld)?;
                    self.vertices.merge(trans, vid_l_new, vid_l, vid_b1r)?;
                    self.vertices.merge(trans, vid_r_new, vid_b1l, vid_r)?;
                    self.attributes
                        .merge_vertex_attributes(trans, vid_l_new, vid_l, vid_b1r)?;
                    self.attributes
                        .merge_vertex_attributes(trans, vid_r_new, vid_b1l, vid_r)?;
                    self.attributes
                        .merge_edge_attributes(trans, eid_new, eid_l, eid_r)?;
                }
            }
            Ok(())
        });
    }
}

/// 2-unsews
impl<T: CoordsFloat> CMap3<T> {
    /// 2-unsew transactional operation.
    pub(crate) fn two_unsew(&self, trans: &mut Transaction, ld: DartIdType) -> CMapResult<()> {
        let rd = self.betas[(2, ld)].read(trans)?;
        let b1ld = self.betas[(1, ld)].read(trans)?;
        let b1rd = self.betas[(1, rd)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1ld == NULL_DART_ID, b1rd == NULL_DART_ID) {
            (true, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, ld)?;
                // update the topology
                self.betas.two_unlink_core(trans, ld)?;
                // split attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .try_split_edge_attributes(trans, ld, rd, eid_old)?;
            }
            (true, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, ld)?;
                let vid_l = self.vertex_id_transac(trans, ld)?;
                // update the topology
                self.betas.two_unlink_core(trans, ld)?;
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .try_split_edge_attributes(trans, ld, rd, eid_old)?;
                let (vid_l_newl, vid_l_newr) = (
                    self.vertex_id_transac(trans, ld)?,
                    self.vertex_id_transac(trans, b1rd)?,
                );
                self.attributes
                    .try_split_vertex_attributes(trans, vid_l_newl, vid_l_newr, vid_l)?;
            }
            (false, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, ld)?;
                let vid_r = self.vertex_id_transac(trans, rd)?;
                // update the topology
                self.betas.two_unlink_core(trans, ld)?;
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .try_split_edge_attributes(trans, ld, rd, eid_old)?;
                let (vid_r_newl, vid_r_newr) = (
                    self.vertex_id_transac(trans, b1ld)?,
                    self.vertex_id_transac(trans, rd)?,
                );
                self.attributes
                    .try_split_vertex_attributes(trans, vid_r_newl, vid_r_newr, vid_r)?;
            }
            (false, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, ld)?;
                let vid_l = self.vertex_id_transac(trans, ld)?;
                let vid_r = self.vertex_id_transac(trans, rd)?;
                // update the topology
                self.betas.two_unlink_core(trans, ld)?;
                // split vertices & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .try_split_edge_attributes(trans, ld, rd, eid_old)?;
                let (vid_l_newl, vid_l_newr) = (
                    self.vertex_id_transac(trans, ld)?,
                    self.vertex_id_transac(trans, b1rd)?,
                );
                let (vid_r_newl, vid_r_newr) = (
                    self.vertex_id_transac(trans, b1ld)?,
                    self.vertex_id_transac(trans, rd)?,
                );
                self.attributes
                    .try_split_vertex_attributes(trans, vid_l_newl, vid_l_newr, vid_l)?;
                self.attributes
                    .try_split_vertex_attributes(trans, vid_r_newl, vid_r_newr, vid_r)?;
            }
        }
        Ok(())
    }

    /// 2-unsew operation.
    pub(crate) fn force_two_unsew(&self, ld: DartIdType) {
        atomically(|trans| {
            let rd = self.betas[(2, ld)].read(trans)?;
            let b1ld = self.betas[(1, ld)].read(trans)?;
            let b1rd = self.betas[(1, rd)].read(trans)?;
            // match (is lhs 1-free, is rhs 1-free)
            match (b1ld == NULL_DART_ID, b1rd == NULL_DART_ID) {
                (true, true) => {
                    // fetch IDs before topology update
                    let eid_old = self.edge_id_transac(trans, ld)?;
                    // update the topology
                    self.betas.two_unlink_core(trans, ld)?;
                    // split attributes from the old ID to the new ones
                    // FIXME: VertexIdentifier should be cast to DartIdentifier
                    self.attributes
                        .split_edge_attributes(trans, ld, rd, eid_old)?;
                }
                (true, false) => {
                    // fetch IDs before topology update
                    let eid_old = self.edge_id_transac(trans, ld)?;
                    let vid_l = self.vertex_id_transac(trans, ld)?;
                    // update the topology
                    self.betas.two_unlink_core(trans, ld)?;
                    // split vertex & attributes from the old ID to the new ones
                    // FIXME: VertexIdentifier should be cast to DartIdentifier
                    self.attributes
                        .split_edge_attributes(trans, ld, rd, eid_old)?;
                    let (vid_l_newl, vid_l_newr) = (
                        self.vertex_id_transac(trans, ld)?,
                        self.vertex_id_transac(trans, b1rd)?,
                    );
                    self.attributes
                        .split_vertex_attributes(trans, vid_l_newl, vid_l_newr, vid_l)?;
                }
                (false, true) => {
                    // fetch IDs before topology update
                    let eid_old = self.edge_id_transac(trans, ld)?;
                    let vid_r = self.vertex_id_transac(trans, rd)?;
                    // update the topology
                    self.betas.two_unlink_core(trans, ld)?;
                    // split vertex & attributes from the old ID to the new ones
                    // FIXME: VertexIdentifier should be cast to DartIdentifier
                    self.attributes
                        .split_edge_attributes(trans, ld, rd, eid_old)?;
                    let (vid_r_newl, vid_r_newr) = (
                        self.vertex_id_transac(trans, b1ld)?,
                        self.vertex_id_transac(trans, rd)?,
                    );
                    self.attributes
                        .split_vertex_attributes(trans, vid_r_newl, vid_r_newr, vid_r)?;
                }
                (false, false) => {
                    // fetch IDs before topology update
                    let eid_old = self.edge_id_transac(trans, ld)?;
                    let vid_l = self.vertex_id_transac(trans, ld)?;
                    let vid_r = self.vertex_id_transac(trans, rd)?;
                    // update the topology
                    self.betas.two_unlink_core(trans, ld)?;
                    // split vertices & attributes from the old ID to the new ones
                    // FIXME: VertexIdentifier should be cast to DartIdentifier
                    self.attributes
                        .split_edge_attributes(trans, ld, rd, eid_old)?;
                    let (vid_l_newl, vid_l_newr) = (
                        self.vertex_id_transac(trans, ld)?,
                        self.vertex_id_transac(trans, b1rd)?,
                    );
                    let (vid_r_newl, vid_r_newr) = (
                        self.vertex_id_transac(trans, b1ld)?,
                        self.vertex_id_transac(trans, rd)?,
                    );
                    self.attributes
                        .split_vertex_attributes(trans, vid_l_newl, vid_l_newr, vid_l)?;
                    self.attributes
                        .split_vertex_attributes(trans, vid_r_newl, vid_r_newr, vid_r)?;
                }
            }
            Ok(())
        });
    }
}
