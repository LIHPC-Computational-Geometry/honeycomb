//! (Un)sew and (un)link implementations
//!
//! This module contains code used to implement sew, unsew, link and unlink operations in all
//! dimensions for which they are defined (1, 2) for a [`CMap2`].

// ------ IMPORTS

use crate::{
    AttributeStorage, AttributeUpdate, CMap2, CoordsFloat, DartIdentifier, UnknownAttributeStorage,
    Vertex2, NULL_DART_ID,
};

// ------ CONTENT

// --- (un)sew operations
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
    pub fn one_sew(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        // this operation only makes sense if lhs_dart is associated to a fully defined edge, i.e.
        // its image through beta2 is defined & has a valid associated vertex (we assume the second
        // condition is valid if the first one is)
        // if that is not the case, the sewing operation becomes a linking operation
        let b2lhs_dart_id = self.beta::<2>(lhs_dart_id);
        if b2lhs_dart_id == NULL_DART_ID {
            assert!(
                self.vertices.get(self.vertex_id(rhs_dart_id)).is_some(),
                "{}",
                format!(
                    "No vertex defined on dart {rhs_dart_id}, use `one_link` instead of `one_sew`"
                )
            );
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
    pub fn two_sew(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        let b1lhs_dart_id = self.beta::<1>(lhs_dart_id);
        let b1rhs_dart_id = self.beta::<1>(rhs_dart_id);
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            // trivial case, no update needed
            (true, true) => {
                assert!(
                    self.vertices.get(self.vertex_id(lhs_dart_id)).is_some() | self.vertices.get(self.vertex_id(rhs_dart_id)).is_some(),
                    "{}",
                    format!("No vertices defined on either darts {lhs_dart_id}/{rhs_dart_id} , use `two_link` instead of `two_sew`")
                );
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
    ///
    pub fn one_unsew(&mut self, lhs_dart_id: DartIdentifier) {
        let b2lhs_dart_id = self.beta::<2>(lhs_dart_id);
        if b2lhs_dart_id == NULL_DART_ID {
            self.one_unlink(lhs_dart_id);
        } else {
            // read current values / remove old ones
            let rhs_dart_id = self.beta::<1>(lhs_dart_id);
            // we only need to remove a single vertex since we're unlinking
            let vertex = self.remove_vertex(self.vertex_id(rhs_dart_id)).unwrap();
            let (v1, v2) = Vertex2::split(vertex);
            // update the topology
            self.one_unlink(lhs_dart_id);
            // reinsert correct values
            let _ = self.replace_vertex(self.vertex_id(b2lhs_dart_id), v1);
            let _ = self.replace_vertex(self.vertex_id(rhs_dart_id), v2);
        }
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
    ///
    pub fn two_unsew(&mut self, lhs_dart_id: DartIdentifier) {
        let rhs_dart_id = self.beta::<2>(lhs_dart_id);
        let b1lhs_dart_id = self.beta::<1>(lhs_dart_id);
        let b1rhs_dart_id = self.beta::<1>(rhs_dart_id);
        // match (is lhs 1-free, is rhs 1-free)
        match (b1lhs_dart_id == NULL_DART_ID, b1rhs_dart_id == NULL_DART_ID) {
            (true, true) => self.two_unlink(lhs_dart_id),
            (true, false) => {
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                let rhs_vertex = self.remove_vertex(rhs_vid_old).unwrap();
                let (v1, v2) = Vertex2::split(rhs_vertex);
                self.two_unlink(lhs_dart_id);
                self.insert_vertex(self.vertex_id(b1lhs_dart_id), v1);
                self.insert_vertex(self.vertex_id(rhs_dart_id), v2);
            }
            (false, true) => {
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let lhs_vertex = self.remove_vertex(lhs_vid_old).unwrap();
                let (v1, v2) = Vertex2::split(lhs_vertex);
                self.two_unlink(lhs_dart_id);
                self.insert_vertex(self.vertex_id(lhs_dart_id), v1);
                self.insert_vertex(self.vertex_id(b1rhs_dart_id), v2);
            }
            (false, false) => {
                let lhs_vid_old = self.vertex_id(lhs_dart_id);
                let rhs_vid_old = self.vertex_id(rhs_dart_id);
                let lhs_vertex = self.remove_vertex(lhs_vid_old).unwrap();
                let rhs_vertex = self.remove_vertex(rhs_vid_old).unwrap();
                self.two_unlink(lhs_dart_id);
                let (rhs_v1, rhs_v2) = Vertex2::split(rhs_vertex);
                let (lhs_v1, lhs_v2) = Vertex2::split(lhs_vertex);

                // short version: not all i-unsews create separate i-cells
                self.insert_vertex(self.vertex_id(b1lhs_dart_id), rhs_v1);
                let _ = self.replace_vertex(self.vertex_id(rhs_dart_id), rhs_v2);
                // same
                self.insert_vertex(self.vertex_id(lhs_dart_id), lhs_v1);
                let _ = self.replace_vertex(self.vertex_id(b1rhs_dart_id), lhs_v2);
            }
        }
    }
}

// --- (un)link operations
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
    pub fn one_link(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        // we could technically overwrite the value, but these assertions
        // makes it easier to assert algorithm correctness
        assert!(self.is_i_free::<1>(lhs_dart_id));
        assert!(self.is_i_free::<0>(rhs_dart_id));
        self.betas[lhs_dart_id as usize][1] = rhs_dart_id; // set beta_1(lhs_dart) to rhs_dart
        self.betas[rhs_dart_id as usize][0] = lhs_dart_id; // set beta_0(rhs_dart) to lhs_dart
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
    ///
    pub fn two_link(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        // we could technically overwrite the value, but these assertions
        // make it easier to assert algorithm correctness
        assert!(self.is_i_free::<2>(lhs_dart_id));
        assert!(self.is_i_free::<2>(rhs_dart_id));
        self.betas[lhs_dart_id as usize][2] = rhs_dart_id; // set beta_2(lhs_dart) to rhs_dart
        self.betas[rhs_dart_id as usize][2] = lhs_dart_id; // set beta_2(rhs_dart) to lhs_dart
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
    pub fn one_unlink(&mut self, lhs_dart_id: DartIdentifier) {
        let rhs_dart_id = self.beta::<1>(lhs_dart_id); // fetch id of beta_1(lhs_dart)
        self.betas[lhs_dart_id as usize][1] = 0; // set beta_1(lhs_dart) to NullDart
        self.betas[rhs_dart_id as usize][0] = 0; // set beta_0(rhs_dart) to NullDart
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
    pub fn two_unlink(&mut self, lhs_dart_id: DartIdentifier) {
        let rhs_dart_id = self.beta::<2>(lhs_dart_id); // fetch id of beta_2(lhs_dart)
        self.betas[lhs_dart_id as usize][2] = 0; // set beta_2(dart) to NullDart
        self.betas[rhs_dart_id as usize][2] = 0; // set beta_2(beta_2(dart)) to NullDart
    }
}
