//! (Un)sew and (un)link implementations
//!
//! This module contains code used to implement sew, unsew, link and unlink operations in all
//! dimensions for which they are defined (1, 2) for a [`CMap2`].

// ------ IMPORTS

use crate::prelude::{CMap2, DartIdentifier, NULL_DART_ID};
use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    geometry::CoordsFloat,
};
use stm::{atomically, StmError, Transaction};

// ------ CONTENT

/// **Sew and unsew operations**
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
    /// # Panics
    ///
    /// The method may panic if the two darts are not 1-sewable.
    ///
    pub fn one_sew(&self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        // this operation only makes sense if lhs_dart is associated to a fully defined edge, i.e.
        // its image through beta2 is defined & has a valid associated vertex (we assume the second
        // condition is valid if the first one is)
        // if that is not the case, the sewing operation becomes a linking operation
        let b2lhs_dart_id = self.beta::<2>(lhs_dart_id);
        if b2lhs_dart_id == NULL_DART_ID {
            self.one_link(lhs_dart_id, rhs_dart_id);
        } else {
            // fetch vertices ID before topology update
            let b2lhs_vid_old = self.vertex_id(b2lhs_dart_id);
            let rhs_vid_old = self.vertex_id(rhs_dart_id);
            // update the topology
            self.one_link(lhs_dart_id, rhs_dart_id);
            // merge vertices & attributes from the old IDs to the new one
            // FIXME: VertexIdentifier should be cast to DartIdentifier
            self.vertices
                .merge(self.vertex_id(rhs_dart_id), b2lhs_vid_old, rhs_vid_old);
            self.attributes.merge_vertex_attributes(
                self.vertex_id(rhs_dart_id),
                b2lhs_vid_old,
                rhs_vid_old,
            );
        }
    }

    /// Atomically 1-sew two darts.
    pub fn atomically_one_sew(&self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        atomically(|trans| {
            let b2lhs_dart_id = self.betas[lhs_dart_id as usize][2].read(trans)?;
            if b2lhs_dart_id == NULL_DART_ID {
                self.one_link_core(trans, lhs_dart_id, rhs_dart_id)
            } else {
                let b2lhs_vid_old = self.vertex_id_transac(trans, b2lhs_dart_id)?;
                let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;

                self.one_link_core(trans, lhs_dart_id, rhs_dart_id)?;

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
        });
    }

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
    /// # Panics
    ///
    /// The method may panic if:
    /// - the two darts are not 2-sewable,
    /// - the method cannot resolve orientation issues.
    ///
    pub fn two_sew(&self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        let b1lhs_dart_id = self.beta::<1>(lhs_dart_id);
        let b1rhs_dart_id = self.beta::<1>(rhs_dart_id);
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => {
                self.two_link(lhs_dart_id, rhs_dart_id);
            }
            // update vertex associated to b1rhs/lhs
            (true, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id(lhs_dart_id);
                let rhs_eid_old = self.edge_id(b1rhs_dart_id);
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let b1rhs_vid_old = self.vertex_id(b1rhs_dart_id);
                // update the topology
                self.two_link(lhs_dart_id, rhs_dart_id);
                // merge vertices & attributes from the old IDs to the new one
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.vertices
                    .merge(self.vertex_id(lhs_dart_id), lhs_vid_old, b1rhs_vid_old);
                self.attributes.merge_vertex_attributes(
                    self.vertex_id(lhs_dart_id),
                    lhs_vid_old,
                    b1rhs_vid_old,
                );
                self.attributes.merge_edge_attributes(
                    self.edge_id(lhs_dart_id),
                    lhs_eid_old,
                    rhs_eid_old,
                );
            }
            // update vertex associated to b1lhs/rhs
            (false, true) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id(lhs_dart_id);
                let rhs_eid_old = self.edge_id(b1rhs_dart_id);
                let b1lhs_vid_old = self.vertex_id(b1lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                // update the topology
                self.two_link(lhs_dart_id, rhs_dart_id);
                // merge vertices & attributes from the old IDs to the new one
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.vertices
                    .merge(self.vertex_id(rhs_dart_id), b1lhs_vid_old, rhs_vid_old);
                self.attributes.merge_vertex_attributes(
                    self.vertex_id(rhs_dart_id),
                    b1lhs_vid_old,
                    rhs_vid_old,
                );
                self.attributes.merge_edge_attributes(
                    self.edge_id(lhs_dart_id),
                    lhs_eid_old,
                    rhs_eid_old,
                );
            }
            // update both vertices making up the edge
            (false, false) => {
                // fetch vertices ID before topology update
                let lhs_eid_old = self.edge_id(lhs_dart_id);
                let rhs_eid_old = self.edge_id(b1rhs_dart_id);
                // (lhs/b1rhs) vertex
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let b1rhs_vid_old = self.vertex_id(b1rhs_dart_id);
                // (b1lhs/rhs) vertex
                let b1lhs_vid_old = self.vertex_id(b1lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);

                // check orientation
                // FIXME: using `get` is suboptimal because read ops imply a copy in our collections
                // FIXME: maybe we should directly read into the storage instead of using its API
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
                    // FIXME: should we crash in case of inconsistent orientation?
                    assert!(
                        lhs_vector.dot(&rhs_vector) < T::zero(),
                        "{}",
                        format!("Dart {lhs_dart_id} and {rhs_dart_id} do not have consistent orientation for 2-sewing"),
                    );
                };

                // update the topology
                self.two_link(lhs_dart_id, rhs_dart_id);
                // merge vertices & attributes from the old IDs to the new one
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.vertices
                    .merge(self.vertex_id(lhs_dart_id), lhs_vid_old, b1rhs_vid_old);
                self.vertices
                    .merge(self.vertex_id(rhs_dart_id), b1lhs_vid_old, rhs_vid_old);
                self.attributes.merge_vertex_attributes(
                    self.vertex_id(lhs_dart_id),
                    lhs_vid_old,
                    b1rhs_vid_old,
                );
                self.attributes.merge_vertex_attributes(
                    self.vertex_id(rhs_dart_id),
                    b1lhs_vid_old,
                    rhs_vid_old,
                );
                self.attributes.merge_edge_attributes(
                    self.edge_id(lhs_dart_id),
                    lhs_eid_old,
                    rhs_eid_old,
                );
            }
        }
    }

    #[allow(clippy::missing_panics_doc)]
    /// Atomically 2-sew two darts.
    #[allow(clippy::too_many_lines)]
    pub fn atomically_two_sew(&self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        atomically(|trans| {
            let b1lhs_dart_id = self.betas[lhs_dart_id as usize][1].read(trans)?;
            let b1rhs_dart_id = self.betas[rhs_dart_id as usize][1].read(trans)?;
            // match (is lhs 1-free, is rhs 1-free)
            match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
                // trivial case, no update needed
                (true, true) => {
                    self.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                }
                // update vertex associated to b1rhs/lhs
                (true, false) => {
                    // fetch vertices ID before topology update
                    let lhs_eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                    let rhs_eid_old = self.edge_id_transac(trans, b1rhs_dart_id)?;
                    let lhs_vid_old = self.vertex_id_transac(trans, lhs_dart_id)?;
                    let b1rhs_vid_old = self.vertex_id_transac(trans, b1rhs_dart_id)?;
                    // update the topology
                    self.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                    // merge vertices & attributes from the old IDs to the new one
                    // FIXME: VertexIdentifier should be cast to DartIdentifier
                    self.vertices.merge_core(
                        trans,
                        self.vertex_id(lhs_dart_id),
                        lhs_vid_old,
                        b1rhs_vid_old,
                    )?;
                    self.attributes.merge_vertex_attributes_transac(
                        trans,
                        self.vertex_id(lhs_dart_id),
                        lhs_vid_old,
                        b1rhs_vid_old,
                    )?;
                    self.attributes.merge_edge_attributes_transac(
                        trans,
                        self.edge_id(lhs_dart_id),
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
                    self.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                    // merge vertices & attributes from the old IDs to the new one
                    // FIXME: VertexIdentifier should be cast to DartIdentifier
                    self.vertices.merge_core(
                        trans,
                        self.vertex_id(rhs_dart_id),
                        b1lhs_vid_old,
                        rhs_vid_old,
                    )?;
                    self.attributes.merge_vertex_attributes_transac(
                        trans,
                        self.vertex_id(rhs_dart_id),
                        b1lhs_vid_old,
                        rhs_vid_old,
                    )?;
                    self.attributes.merge_edge_attributes_transac(
                        trans,
                        self.edge_id(lhs_dart_id),
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
                    self.two_link_core(trans, lhs_dart_id, rhs_dart_id)?;
                    // merge vertices & attributes from the old IDs to the new one
                    // FIXME: VertexIdentifier should be cast to DartIdentifier
                    self.vertices.merge_core(
                        trans,
                        self.vertex_id(lhs_dart_id),
                        lhs_vid_old,
                        b1rhs_vid_old,
                    )?;
                    self.vertices.merge_core(
                        trans,
                        self.vertex_id(rhs_dart_id),
                        b1lhs_vid_old,
                        rhs_vid_old,
                    )?;
                    self.attributes.merge_vertex_attributes_transac(
                        trans,
                        self.vertex_id(lhs_dart_id),
                        lhs_vid_old,
                        b1rhs_vid_old,
                    )?;
                    self.attributes.merge_vertex_attributes_transac(
                        trans,
                        self.vertex_id(rhs_dart_id),
                        b1lhs_vid_old,
                        rhs_vid_old,
                    )?;
                    self.attributes.merge_edge_attributes_transac(
                        trans,
                        self.edge_id(lhs_dart_id),
                        lhs_eid_old,
                        rhs_eid_old,
                    )?;
                }
            }
            Ok(())
        });
    }

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
    /// # Panics
    ///
    /// The method may panic if there's a missing attribute at the splitting step. While the
    /// implementation could fall back to a simple unlink operation, it probably should have been
    /// called by the user, instead of unsew, in the first place.
    pub fn one_unsew(&self, lhs_dart_id: DartIdentifier) {
        let b2lhs_dart_id = self.beta::<2>(lhs_dart_id);
        if b2lhs_dart_id == NULL_DART_ID {
            self.one_unlink(lhs_dart_id);
        } else {
            // fetch IDs before topology update
            let rhs_dart_id = self.beta::<1>(lhs_dart_id);
            let vid_old = self.vertex_id(rhs_dart_id);
            // update the topology
            self.one_unlink(lhs_dart_id);
            // split vertices & attributes from the old ID to the new ones
            // FIXME: VertexIdentifier should be cast to DartIdentifier
            self.vertices.split(
                self.vertex_id(b2lhs_dart_id),
                self.vertex_id(rhs_dart_id),
                vid_old,
            );
            self.attributes.split_vertex_attributes(
                self.vertex_id(b2lhs_dart_id),
                self.vertex_id(rhs_dart_id),
                vid_old,
            );
        }
    }

    /// Atomically 1-unsew two darts.
    pub fn atomically_one_unsew(&self, lhs_dart_id: DartIdentifier) {
        atomically(|trans| {
            let b2lhs_dart_id = self.betas[lhs_dart_id as usize][2].read(trans)?;
            if b2lhs_dart_id == NULL_DART_ID {
                self.one_unlink_core(trans, lhs_dart_id)?;
            } else {
                // fetch IDs before topology update
                let rhs_dart_id = self.betas[lhs_dart_id as usize][1].read(trans)?;
                let vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
                // update the topology
                self.one_unlink_core(trans, lhs_dart_id)?;
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
        });
    }

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
    /// # Panics
    ///
    /// The method may panic if there's a missing attribute at the splitting step. While the
    /// implementation could fall back to a simple unlink operation, it probably should have been
    /// called by the user, instead of unsew, in the first place.
    pub fn two_unsew(&self, lhs_dart_id: DartIdentifier) {
        let rhs_dart_id = self.beta::<2>(lhs_dart_id);
        let b1lhs_dart_id = self.beta::<1>(lhs_dart_id);
        let b1rhs_dart_id = self.beta::<1>(rhs_dart_id);
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            (true, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id(lhs_dart_id);
                // update the topology
                self.two_unlink(lhs_dart_id);
                // split attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .split_edge_attributes(lhs_dart_id, rhs_dart_id, eid_old);
            }
            (true, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id(lhs_dart_id);
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                // update the topology
                self.two_unlink(lhs_dart_id);
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .split_edge_attributes(lhs_dart_id, rhs_dart_id, eid_old);
                let (new_lv_lhs, new_lv_rhs) =
                    (self.vertex_id(lhs_dart_id), self.vertex_id(b1rhs_dart_id));
                self.attributes
                    .split_vertex_attributes(new_lv_lhs, new_lv_rhs, lhs_vid_old);
            }
            (false, true) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id(lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                // update the topology
                self.two_unlink(lhs_dart_id);
                // split vertex & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .split_edge_attributes(lhs_dart_id, rhs_dart_id, eid_old);
                let (new_rv_lhs, new_rv_rhs) =
                    (self.vertex_id(b1lhs_dart_id), self.vertex_id(rhs_dart_id));
                self.attributes
                    .split_vertex_attributes(new_rv_lhs, new_rv_rhs, rhs_vid_old);
            }
            (false, false) => {
                // fetch IDs before topology update
                let eid_old = self.edge_id(lhs_dart_id);
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                // update the topology
                self.two_unlink(lhs_dart_id);
                // split vertices & attributes from the old ID to the new ones
                // FIXME: VertexIdentifier should be cast to DartIdentifier
                self.attributes
                    .split_edge_attributes(lhs_dart_id, rhs_dart_id, eid_old);
                let (new_lv_lhs, new_lv_rhs) =
                    (self.vertex_id(lhs_dart_id), self.vertex_id(b1rhs_dart_id));
                let (new_rv_lhs, new_rv_rhs) =
                    (self.vertex_id(b1lhs_dart_id), self.vertex_id(rhs_dart_id));
                self.attributes
                    .split_vertex_attributes(new_lv_lhs, new_lv_rhs, lhs_vid_old);
                self.attributes
                    .split_vertex_attributes(new_rv_lhs, new_rv_rhs, rhs_vid_old);
            }
        }
    }

    /// Atomically 2-unsew two darts.
    pub fn atomically_two_unsew(&self, lhs_dart_id: DartIdentifier) {
        atomically(|trans| {
            let rhs_dart_id = self.betas[lhs_dart_id as usize][2].read(trans)?;
            let b1lhs_dart_id = self.betas[lhs_dart_id as usize][1].read(trans)?;
            let b1rhs_dart_id = self.betas[rhs_dart_id as usize][1].read(trans)?;
            // match (is lhs 1-free, is rhs 1-free)
            match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
                (true, true) => {
                    // fetch IDs before topology update
                    let eid_old = self.edge_id_transac(trans, lhs_dart_id)?;
                    // update the topology
                    self.two_unlink_core(trans, lhs_dart_id)?;
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
                    self.two_unlink_core(trans, lhs_dart_id)?;
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
                    self.two_unlink_core(trans, lhs_dart_id)?;
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
                    self.two_unlink_core(trans, lhs_dart_id)?;
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
        });
    }
}

/// **Link and unlink operations**
impl<T: CoordsFloat> CMap2<T> {
    /// 1-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>1</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s). The *β<sub>0</sub>* function is also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if `lhs_dart_id` isn't 1-free or `rhs_dart_id` isn't 0-free.
    ///
    pub fn one_link(&self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        atomically(|trans| self.one_link_core(trans, lhs_dart_id, rhs_dart_id));
    }

    /// 2-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>2</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` or `rhs_dart_id` isn't 2-free.
    pub fn two_link(&self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        atomically(|trans| self.two_link_core(trans, lhs_dart_id, rhs_dart_id));
    }

    /// 1-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>1</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s). The *β<sub>0</sub>* function is
    /// also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 1-free.
    pub fn one_unlink(&self, lhs_dart_id: DartIdentifier) {
        atomically(|trans| self.one_unlink_core(trans, lhs_dart_id));
    }

    /// 2-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>2</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 2-free.
    pub fn two_unlink(&self, lhs_dart_id: DartIdentifier) {
        atomically(|trans| self.two_unlink_core(trans, lhs_dart_id));
    }
}

#[doc(hidden)]
/// **Link and unlink core operations**
impl<T: CoordsFloat> CMap2<T> {
    /// 1-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>1</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s). The *β<sub>0</sub>* function is also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if `lhs_dart_id` isn't 1-free or `rhs_dart_id` isn't 0-free.
    ///
    pub(crate) fn one_link_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdentifier,
        rhs_dart_id: DartIdentifier,
    ) -> Result<(), StmError> {
        // we could technically overwrite the value, but these assertions
        // makes it easier to assert algorithm correctness
        assert!(self.is_i_free::<1>(lhs_dart_id));
        assert!(self.is_i_free::<0>(rhs_dart_id));
        // set beta_1(lhs_dart) to rhs_dart
        self.betas[lhs_dart_id as usize][1].write(trans, rhs_dart_id)?;
        // set beta_0(rhs_dart) to lhs_dart
        self.betas[rhs_dart_id as usize][0].write(trans, lhs_dart_id)?;
        Ok(())
    }

    /// 2-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>2</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` or `rhs_dart_id` isn't 2-free.
    pub(crate) fn two_link_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdentifier,
        rhs_dart_id: DartIdentifier,
    ) -> Result<(), StmError> {
        // we could technically overwrite the value, but these assertions
        // make it easier to assert algorithm correctness
        assert!(self.is_i_free::<2>(lhs_dart_id));
        assert!(self.is_i_free::<2>(rhs_dart_id));
        // set beta_2(lhs_dart) to rhs_dart
        self.betas[lhs_dart_id as usize][2].write(trans, rhs_dart_id)?;
        // set beta_2(rhs_dart) to lhs_dart
        self.betas[rhs_dart_id as usize][2].write(trans, lhs_dart_id)?;
        Ok(())
    }

    /// 1-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>1</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s). The *β<sub>0</sub>* function is
    /// also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 1-free.
    pub(crate) fn one_unlink_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdentifier,
    ) -> Result<(), StmError> {
        // set beta_1(lhs_dart) to NullDart
        let rhs_dart_id = self.betas[lhs_dart_id as usize][1].replace(trans, NULL_DART_ID)?;
        assert_ne!(rhs_dart_id, NULL_DART_ID);
        // set beta_0(rhs_dart) to NullDart
        self.betas[rhs_dart_id as usize][0].write(trans, NULL_DART_ID)?;
        Ok(())
    }

    /// 2-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>2</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 2-free.
    pub(crate) fn two_unlink_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdentifier,
    ) -> Result<(), StmError> {
        // set beta_2(dart) to NullDart
        let rhs_dart_id = self.betas[lhs_dart_id as usize][2].replace(trans, NULL_DART_ID)?;
        assert_ne!(rhs_dart_id, NULL_DART_ID);
        // set beta_2(beta_2(dart)) to NullDart
        self.betas[rhs_dart_id as usize][2].write(trans, NULL_DART_ID)?;
        Ok(())
    }
}
