//! 2D sew implementations

use stm::{atomically, StmError, Transaction};

use crate::{
    attributes::AttributeStorage,
    cmap::{CMap2, DartIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

/// 2-sews
impl<T: CoordsFloat> CMap2<T> {
    /// 2-sew operation.
    ///
    /// This operation corresponds to *coherently linking* two darts via the *β<sub>2</sub>*
    /// function. For a thorough explanation of this operation (and implied hypothesis &
    /// consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    /// - `policy: SewPolicy` -- Geometrical sewing policy to follow.
    ///
    /// After the sewing operation, these darts will verify
    /// *β<sub>2</sub>(`lhs_dart`) = `rhs_dart`* and *β<sub>2</sub>(`rhs_dart`) = `lhs_dart`*.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    ///
    /// The policy in case of failure can be defined through the transaction, using
    /// `Transaction::with_control` for construction.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the two darts are not 2-sewable,
    /// - the method cannot resolve orientation issues.
    #[allow(clippy::too_many_lines)]
    pub fn two_sew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let b1lhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
        let b1rhs_dart_id = self.betas[(1, rhs_dart_id)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => {
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
            }
            // update vertex associated to b1rhs/lhs
            (true, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_eid_old = self.edge_id_transac(trans, b1rhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let b1rhs_vid_old = self.vertex_id_transac(trans, b1rhs_dart_id)?;
                // update the topology
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                // merge vertices & attributes from the old IDs to the new one
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                let lhs_vid_new = self.vertex_id_transac(trans, lhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                self.vertices
                    .merge_core(trans, lhs_vid_new, lhs_vid_old, b1rhs_vid_old)?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    lhs_vid_new,
                    lhs_vid_old,
                    b1rhs_vid_old,
                )?;
                self.attributes.merge_edge_attributes_transac(
                    trans,
                    eid_new,
                    lhs_eid_old,
                    rhs_eid_old,
                )?;
            }
            // update vertex associated to b1lhs/rhs
            (false, true) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_eid_old = self.edge_id_transac(trans, b1rhs_dart_id)?;
                let b1lhs_vid_old = self.vertex_id_transac(trans, b1lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                // merge vertices & attributes from the old IDs to the new one
                let rhs_vid_new = self.vertex_id_transac(trans, rhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                self.vertices
                    .merge_core(trans, rhs_vid_new, b1lhs_vid_old, rhs_vid_old)?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    rhs_vid_new,
                    b1lhs_vid_old,
                    rhs_vid_old,
                )?;
                self.attributes.merge_edge_attributes_transac(
                    trans,
                    eid_new,
                    lhs_eid_old,
                    rhs_eid_old,
                )?;
            }
            // update both vertices making up the edge
            (false, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_eid_old = self.edge_id_transac(trans, b1rhs_dart_id)?;
                // (lhs/b1rhs) vertex
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let b1rhs_vid_old = self.vertex_id_transac(trans, b1rhs_dart_id)?;
                // (b1lhs/rhs) vertex
                let b1lhs_vid_old = self.vertex_id_transac(trans, b1lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;

                // check orientation
                #[rustfmt::skip]
                    if let (
                        Some(l_vertex), Some(b1r_vertex), // (lhs/b1rhs) vertices
                        Some(b1l_vertex), Some(r_vertex), // (b1lhs/rhs) vertices
                    ) = (
                        self.vertices.get_core(trans, &lhs_vid_old)?, self.vertices.get_core(trans, &b1rhs_vid_old)?,// (lhs/b1rhs)
                        self.vertices.get_core(trans, &b1lhs_vid_old)?, self.vertices.get_core(trans, &rhs_vid_old)? // (b1lhs/rhs)
                    )
                    {
                        let lhs_vector = b1l_vertex - l_vertex;
                        let rhs_vector = b1r_vertex - r_vertex;
                        // dot product should be negative if the two darts have opposite direction
                        // we could also put restriction on the angle made by the two darts to prevent
                        // drastic deformation
                        assert!(
                            lhs_vector.dot(&rhs_vector) < T::zero(),
                            "{}",
                            format!("Dart {lhs_dart_id} and {rhs_dart_id} do not have consistent orientation for 2-sewing"),
                        );
                    };

                // update the topology
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                // merge vertices & attributes from the old IDs to the new one
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                let lhs_vid_new = self.vertex_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_new = self.vertex_id_transac(trans, rhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                self.vertices
                    .merge_core(trans, lhs_vid_new, lhs_vid_old, b1rhs_vid_old)?;
                self.vertices
                    .merge_core(trans, rhs_vid_new, b1lhs_vid_old, rhs_vid_old)?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    lhs_vid_new,
                    lhs_vid_old,
                    b1rhs_vid_old,
                )?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    rhs_vid_new,
                    b1lhs_vid_old,
                    rhs_vid_old,
                )?;
                self.attributes.merge_edge_attributes_transac(
                    trans,
                    eid_new,
                    lhs_eid_old,
                    rhs_eid_old,
                )?;
            }
        }
        Ok(())
    }

    /// 2-sew two darts.
    ///
    /// This variant is equivalent to `two_sew`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_two_sew(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        atomically(|trans| self.two_sew(trans, lhs_dart_id, rhs_dart_id));
    }

    /// Attempt to 2-sew two darts.
    ///
    /// # Errors
    ///
    /// This method will fail, returning an error, if:
    /// - the transaction cannot be completed
    /// - one (or more) attribute merge fails
    ///
    /// The returned error can be used in conjunction with transaction control to avoid any
    /// modifications in case of failure at attribute level. The user can then choose, through its
    /// transaction control policy, to retry or abort as he wishes.
    #[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
    pub fn try_two_sew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let b1lhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
        let b1rhs_dart_id = self.betas[(1, rhs_dart_id)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => {
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
            }
            // update vertex associated to b1rhs/lhs
            (true, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_eid_old = self.edge_id_transac(trans, b1rhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let b1rhs_vid_old = self.vertex_id_transac(trans, b1rhs_dart_id)?;
                // update the topology
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                // merge vertices & attributes from the old IDs to the new one
                // TODO: these should be attempts, only succeding if splitting a value
                let lhs_vid_new = self.vertex_id_transac(trans, lhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                self.vertices
                    .merge_core(trans, lhs_vid_new, lhs_vid_old, b1rhs_vid_old)?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    lhs_vid_new,
                    lhs_vid_old,
                    b1rhs_vid_old,
                )?;
                self.attributes.merge_edge_attributes_transac(
                    trans,
                    eid_new,
                    lhs_eid_old,
                    rhs_eid_old,
                )?;
            }
            // update vertex associated to b1lhs/rhs
            (false, true) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_eid_old = self.edge_id_transac(trans, b1rhs_dart_id)?;
                let b1lhs_vid_old = self.vertex_id_transac(trans, b1lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                // merge vertices & attributes from the old IDs to the new one
                // TODO: these should be attempts, only succeding if splitting a value
                let rhs_vid_new = self.vertex_id_transac(trans, rhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                self.vertices
                    .merge_core(trans, rhs_vid_new, b1lhs_vid_old, rhs_vid_old)?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    rhs_vid_new,
                    b1lhs_vid_old,
                    rhs_vid_old,
                )?;
                self.attributes.merge_edge_attributes_transac(
                    trans,
                    eid_new,
                    lhs_eid_old,
                    rhs_eid_old,
                )?;
            }
            // update both vertices making up the edge
            (false, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_eid_old = self.edge_id_transac(trans, b1rhs_dart_id)?;
                // (lhs/b1rhs) vertex
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let b1rhs_vid_old = self.vertex_id_transac(trans, b1rhs_dart_id)?;
                // (b1lhs/rhs) vertex
                let b1lhs_vid_old = self.vertex_id_transac(trans, b1lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;

                // check orientation
                #[rustfmt::skip]
                    if let (
                        Some(l_vertex), Some(b1r_vertex), // (lhs/b1rhs) vertices
                        Some(b1l_vertex), Some(r_vertex), // (b1lhs/rhs) vertices
                    ) = (
                        self.vertices.get(lhs_vid_old), self.vertices.get(b1rhs_vid_old),// (lhs/b1rhs)
                        self.vertices.get(b1lhs_vid_old), self.vertices.get(rhs_vid_old) // (b1lhs/rhs)
                    )
                    {
                        let lhs_vector = b1l_vertex - l_vertex;
                        let rhs_vector = b1r_vertex - r_vertex;
                        // dot product should be negative if the two darts have opposite direction
                        // we could also put restriction on the angle made by the two darts to prevent
                        // drastic deformation
                        assert!(
                            lhs_vector.dot(&rhs_vector) < T::zero(),
                            "{}",
                            format!("Dart {lhs_dart_id} and {rhs_dart_id} do not have consistent orientation for 2-sewing"),
                        );
                    };

                // update the topology
                self.betas.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                // merge vertices & attributes from the old IDs to the new one
                // TODO: these should be attempts, only succeding if splitting a value
                let lhs_vid_new = self.vertex_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_new = self.vertex_id_transac(trans, rhs_dart_id)?;
                let eid_new = self.edge_id_transac(trans, lhs_dart_id)?;
                self.vertices
                    .merge_core(trans, lhs_vid_new, lhs_vid_old, b1rhs_vid_old)?;
                self.vertices
                    .merge_core(trans, rhs_vid_new, b1lhs_vid_old, rhs_vid_old)?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    lhs_vid_new,
                    lhs_vid_old,
                    b1rhs_vid_old,
                )?;
                self.attributes.merge_vertex_attributes_transac(
                    trans,
                    rhs_vid_new,
                    b1lhs_vid_old,
                    rhs_vid_old,
                )?;
                self.attributes.merge_edge_attributes_transac(
                    trans,
                    eid_new,
                    lhs_eid_old,
                    rhs_eid_old,
                )?;
            }
        }
        Ok(())
    }
}

/// 2-unsews
impl<T: CoordsFloat> CMap2<T> {
    /// 2-unsew operation.
    ///
    /// This operation corresponds to *coherently separating* two darts linked via the
    /// *β<sub>2</sub>* function. For a thorough explanation of this operation (and implied
    /// hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to separate.
    /// - `policy: UnsewPolicy` -- Geometrical unsewing policy to follow.
    ///
    /// Note that we do not need to take two darts as arguments since the second dart can be
    /// obtained through the *β<sub>2</sub>* function.
    ///
    /// # Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    ///
    /// The policy in case of failure can be defined through the transaction, using
    /// `Transaction::with_control` for construction.
    ///
    /// # Panics
    ///
    /// The method may panic if there's a missing attribute at the splitting step. While the
    /// implementation could fall back to a simple unlink operation, it probably should have been
    /// called by the user, instead of unsew, in the first place.
    pub fn two_unsew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let rhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        let b1lhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
        let b1rhs_dart_id = self.betas[(1, rhs_dart_id)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            (true, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
            }
            (true, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
                let (new_lv_lhs, new_lv_rhs) = (
                    self.vertex_id_transac(trans, lhs_dart_id)?,
                    self.vertex_id_transac(trans, b1rhs_dart_id)?,
                );
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_lv_lhs,
                    new_lv_rhs,
                    lhs_vid_old,
                )?;
            }
            (false, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
                let (new_rv_lhs, new_rv_rhs) = (
                    self.vertex_id_transac(trans, b1lhs_dart_id)?,
                    self.vertex_id_transac(trans, rhs_dart_id)?,
                );
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_rv_lhs,
                    new_rv_rhs,
                    rhs_vid_old,
                )?;
            }
            (false, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split vertices & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
                let (new_lv_lhs, new_lv_rhs) = (
                    self.vertex_id_transac(trans, lhs_dart_id)?,
                    self.vertex_id_transac(trans, b1rhs_dart_id)?,
                );
                let (new_rv_lhs, new_rv_rhs) = (
                    self.vertex_id_transac(trans, b1lhs_dart_id)?,
                    self.vertex_id_transac(trans, rhs_dart_id)?,
                );
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_lv_lhs,
                    new_lv_rhs,
                    lhs_vid_old,
                )?;
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_rv_lhs,
                    new_rv_rhs,
                    rhs_vid_old,
                )?;
            }
        }
        Ok(())
    }

    /// 2-unsew two darts.
    ///
    /// This variant is equivalent to `two_unsew`, but internally uses a transaction that will
    /// be retried until validated.
    pub fn force_two_unsew(&self, lhs_dart_id: DartIdType) {
        atomically(|trans| self.two_unsew(trans, lhs_dart_id));
    }

    /// Attempt to 2-unsew two darts.
    ///
    /// # Errors
    ///
    /// This method will fail, returning an error, if:
    /// - the transaction cannot be completed
    /// - one (or more) attribute merge fails
    ///
    /// The returned error can be used in conjunction with transaction control to avoid any
    /// modifications in case of failure at attribute level. The user can then choose, through its
    /// transaction control policy, to retry or abort as he wishes.
    pub fn try_two_unsew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let rhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        let b1lhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
        let b1rhs_dart_id = self.betas[(1, rhs_dart_id)].read(trans)?;
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            (true, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
            }
            (true, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
                let (new_lv_lhs, new_lv_rhs) = (
                    self.vertex_id_transac(trans, lhs_dart_id)?,
                    self.vertex_id_transac(trans, b1rhs_dart_id)?,
                );
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_lv_lhs,
                    new_lv_rhs,
                    lhs_vid_old,
                )?;
            }
            (false, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
                let (new_rv_lhs, new_rv_rhs) = (
                    self.vertex_id_transac(trans, b1lhs_dart_id)?,
                    self.vertex_id_transac(trans, rhs_dart_id)?,
                );
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_rv_lhs,
                    new_rv_rhs,
                    rhs_vid_old,
                )?;
            }
            (false, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                self.betas.two_unlink_core(trans, lhs_dart_id)?;
                // split vertices & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes.split_edge_attributes_transac(
                    trans,
                    lhs_dart_id,
                    rhs_dart_id,
                    eid_old,
                )?;
                let (new_lv_lhs, new_lv_rhs) = (
                    self.vertex_id_transac(trans, lhs_dart_id)?,
                    self.vertex_id_transac(trans, b1rhs_dart_id)?,
                );
                let (new_rv_lhs, new_rv_rhs) = (
                    self.vertex_id_transac(trans, b1lhs_dart_id)?,
                    self.vertex_id_transac(trans, rhs_dart_id)?,
                );
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_lv_lhs,
                    new_lv_rhs,
                    lhs_vid_old,
                )?;
                self.attributes.split_vertex_attributes_transac(
                    trans,
                    new_rv_lhs,
                    new_rv_rhs,
                    rhs_vid_old,
                )?;
            }
        }
        Ok(())
    }
}
