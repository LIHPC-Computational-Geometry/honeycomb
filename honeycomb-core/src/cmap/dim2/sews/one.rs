//! 1D sew implementations

use stm::{atomically, StmError, Transaction};

use crate::{
    cmap::{CMap2, DartIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

/// 1-sews
impl<T: CoordsFloat> CMap2<T> {
    /// 1-sew operation.
    ///
    /// This operation corresponds to *coherently linking* two darts via the *β<sub>1</sub>*
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
    /// *β<sub>1</sub>(`lhs_dart`) = `rhs_dart`*. The *β<sub>0</sub>* function is also updated.
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
    /// The method may panic if the two darts are not 1-sewable.
    pub fn one_sew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let b2lhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        if b2lhs_dart_id == NULL_DART_ID {
            self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id)
        } else {
            let b2lhs_vid_old = self.vertex_id_transac(trans, b2lhs_dart_id)?;
            let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;

            self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id)?;

            let new_vid = self.vertex_id_transac(trans, rhs_dart_id)?;

            // FIXME: VertexIdentifier should be cast to DartIdentifier
            self.vertices
                .merge_core(trans, new_vid, b2lhs_vid_old, rhs_vid_old)?;
            self.attributes.merge_vertex_attributes_transac(
                trans,
                new_vid,
                b2lhs_vid_old,
                rhs_vid_old,
            )?;
            Ok(())
        }
    }

    /// 1-sew two darts.
    ///
    /// This variant is equivalent to `one_sew`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_one_sew(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        atomically(|trans| self.one_sew(trans, lhs_dart_id, rhs_dart_id));
    }

    /// Attempt to 1-sew two darts.
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
    pub fn try_one_sew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let b2lhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        if b2lhs_dart_id == NULL_DART_ID {
            self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id)
        } else {
            let b2lhs_vid_old = self.vertex_id_transac(trans, b2lhs_dart_id)?;
            let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;

            self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id)?;

            let new_vid = self.vertex_id_transac(trans, rhs_dart_id)?;

            // TODO: these should be attempts, only succeding if it's a full merge
            self.vertices
                .merge_core(trans, new_vid, b2lhs_vid_old, rhs_vid_old)?;
            self.attributes.merge_vertex_attributes_transac(
                trans,
                new_vid,
                b2lhs_vid_old,
                rhs_vid_old,
            )?;
            Ok(())
        }
    }
}

/// 1-unsews
impl<T: CoordsFloat> CMap2<T> {
    /// 1-unsew operation.
    ///
    /// This operation corresponds to *coherently separating* two darts linked via the
    /// *β<sub>1</sub>* function. For a thorough explanation of this operation (and implied
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
    /// obtained through the *β<sub>1</sub>* function. The *β<sub>0</sub>* function is also updated.
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
    pub fn one_unsew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let b2lhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        if b2lhs_dart_id == NULL_DART_ID {
            self.betas.one_unlink_core(trans, lhs_dart_id)?;
        } else {
            // fetch IDs before topology update
            let rhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
            let vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
            // update the topology
            self.betas.one_unlink_core(trans, lhs_dart_id)?;
            // split vertices & attributes from the old ID to the new ones
            // FIXME: VertexIdentifier should be cast to DartIdentifier
            let (new_lhs, new_rhs) = (
                self.vertex_id_transac(trans, b2lhs_dart_id)?,
                self.vertex_id_transac(trans, rhs_dart_id)?,
            );
            self.vertices.split_core(trans, new_lhs, new_rhs, vid_old)?;
            self.attributes
                .split_vertex_attributes_transac(trans, new_lhs, new_rhs, vid_old)?;
        }
        Ok(())
    }

    /// 1-unsew two darts.
    ///
    /// This variant is equivalent to `one_unsew`, but internally uses a transaction that will
    /// be retried until validated.
    pub fn force_one_unsew(&self, lhs_dart_id: DartIdType) {
        atomically(|trans| self.one_unsew(trans, lhs_dart_id));
    }

    /// Attempt to 1-unsew two darts.
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
    pub fn try_one_unsew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        let b2lhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        if b2lhs_dart_id == NULL_DART_ID {
            self.betas.one_unlink_core(trans, lhs_dart_id)?;
        } else {
            // fetch IDs before topology update
            let rhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
            let vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
            // update the topology
            self.betas.one_unlink_core(trans, lhs_dart_id)?;
            // split vertices & attributes from the old ID to the new ones
            // TODO: these should be attempts, only succeding if splitting a value
            let (new_lhs, new_rhs) = (
                self.vertex_id_transac(trans, b2lhs_dart_id)?,
                self.vertex_id_transac(trans, rhs_dart_id)?,
            );
            self.vertices.split_core(trans, new_lhs, new_rhs, vid_old)?;
            self.attributes
                .split_vertex_attributes_transac(trans, new_lhs, new_rhs, vid_old)?;
        }
        Ok(())
    }
}
