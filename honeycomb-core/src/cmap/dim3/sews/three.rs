//! 3D sew implementations

use stm::{atomically, Transaction};

use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{
        CMap3, CMapResult, DartIdType, EdgeIdType, Orbit3, OrbitPolicy, VertexIdType, NULL_DART_ID,
    },
    prelude::CoordsFloat,
};

/// 3-sews
impl<T: CoordsFloat> CMap3<T> {
    /// 3-sew operation.
    pub(crate) fn three_sew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> CMapResult<()> {
        // using these custom orbits, I can get both dart of all sides, directly ordered
        // for the merges
        let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
        let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);
        let l_face = l_side.clone().min().expect("E: unreachable");
        let r_face = r_side.clone().min().expect("E: unreachable");
        let mut edges: Vec<(EdgeIdType, EdgeIdType)> = Vec::with_capacity(10);
        let mut vertices: Vec<(VertexIdType, VertexIdType)> = Vec::with_capacity(10);

        // read edge + vertex on the b1ld side. if b0ld == NULL, we need to read the left vertex
        for (l, r) in l_side.zip(r_side) {
            edges.push((
                self.edge_id_transac(trans, l)?,
                self.edge_id_transac(trans, r)?,
            ));
            let b1l = self.beta_transac::<1>(trans, l)?;
            // this monster statement is necessary to handle open faces
            vertices.push((
                if b1l == NULL_DART_ID {
                    let b2l = self.beta_transac::<2>(trans, l)?;
                    if b2l == NULL_DART_ID {
                        NULL_DART_ID // (*)
                    } else {
                        self.vertex_id_transac(trans, b2l)?
                    }
                } else {
                    self.vertex_id_transac(trans, b1l)?
                },
                self.vertex_id_transac(trans, r)?,
            ));
            // one more for good measures (aka open faces)
            if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                let b1r = self.beta_transac::<1>(trans, r)?;
                vertices.push((
                    self.vertex_id_transac(trans, l)?,
                    if b1r == NULL_DART_ID {
                        let b2r = self.beta_transac::<2>(trans, r)?;
                        if b2r == NULL_DART_ID {
                            NULL_DART_ID // (*)
                        } else {
                            self.vertex_id_transac(trans, b2r)?
                        }
                    } else {
                        self.vertex_id_transac(trans, b1r)?
                    },
                ));
            }
        }

        // (*): these branch corresponds to incomplete merges (at best),
        //      or incorrect structure (at worst). that's not a problem
        //      because `three_link` will detect inconsistencies
        self.three_link(trans, ld, rd)?;

        // merge face, edge, vertex attributes
        self.attributes
            .try_merge_face_attributes(trans, l_face.min(r_face), l_face, r_face)?;
        for (eid_l, eid_r) in edges {
            if eid_l != eid_r {
                self.attributes
                    .try_merge_edge_attributes(trans, eid_l.min(eid_r), eid_l, eid_r)?;
            }
        }
        for (vid_l, vid_r) in vertices {
            if vid_l != vid_r {
                self.vertices
                    .try_merge(trans, vid_l.min(vid_r), vid_l, vid_r)?;
                self.attributes.try_merge_vertex_attributes(
                    trans,
                    vid_l.min(vid_r),
                    vid_l,
                    vid_r,
                )?;
            }
        }
        Ok(())
    }

    /// 3-sew operation.
    pub(crate) fn force_three_sew(&self, ld: DartIdType, rd: DartIdType) {
        atomically(|trans| {
            // using these custom orbits, I can get both dart of all sides, directly ordered
            // for the merges
            let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
            let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);
            let l_face = l_side.clone().min().expect("E: unreachable");
            let r_face = r_side.clone().min().expect("E: unreachable");
            let mut edges: Vec<(EdgeIdType, EdgeIdType)> = Vec::with_capacity(10);
            let mut vertices: Vec<(VertexIdType, VertexIdType)> = Vec::with_capacity(10);

            // read edge + vertex on the b1ld side. if b0ld == NULL, we need to read the left vertex
            for (l, r) in l_side.zip(r_side) {
                edges.push((
                    self.edge_id_transac(trans, l)?,
                    self.edge_id_transac(trans, r)?,
                ));
                let b1l = self.beta_transac::<1>(trans, l)?;
                // this monster statement is necessary to handle open faces
                vertices.push((
                    if b1l == NULL_DART_ID {
                        let b2l = self.beta_transac::<2>(trans, l)?;
                        if b2l == NULL_DART_ID {
                            NULL_DART_ID // (*)
                        } else {
                            self.vertex_id_transac(trans, b2l)?
                        }
                    } else {
                        self.vertex_id_transac(trans, b1l)?
                    },
                    self.vertex_id_transac(trans, r)?,
                ));
                // one more for good measures (aka open faces)
                if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                    let b1r = self.beta_transac::<1>(trans, r)?;
                    vertices.push((
                        self.vertex_id_transac(trans, l)?,
                        if b1r == NULL_DART_ID {
                            let b2r = self.beta_transac::<2>(trans, r)?;
                            if b2r == NULL_DART_ID {
                                NULL_DART_ID // (*)
                            } else {
                                self.vertex_id_transac(trans, b2r)?
                            }
                        } else {
                            self.vertex_id_transac(trans, b1r)?
                        },
                    ));
                }
            }

            // (*): these branch corresponds to incomplete merges (at best),
            //      or incorrect structure (at worst). that's not a problem
            //      because `three_link` will detect inconsistencies
            self.three_link(trans, ld, rd)?;

            // merge face, edge, vertex attributes
            self.attributes
                .merge_face_attributes(trans, l_face.min(r_face), l_face, r_face)?;
            for (eid_l, eid_r) in edges {
                if eid_l != eid_r {
                    self.attributes
                        .merge_edge_attributes(trans, eid_l.min(eid_r), eid_l, eid_r)?;
                }
            }
            for (vid_l, vid_r) in vertices {
                if vid_l != vid_r {
                    self.vertices.merge(trans, vid_l.min(vid_r), vid_l, vid_r)?;
                    self.attributes.merge_vertex_attributes(
                        trans,
                        vid_l.min(vid_r),
                        vid_l,
                        vid_r,
                    )?;
                }
            }
            Ok(())
        });
    }
}

/// 3-unsews
impl<T: CoordsFloat> CMap3<T> {
    /// 3-unsew operation.
    pub(crate) fn three_unsew(&self, trans: &mut Transaction, ld: DartIdType) -> CMapResult<()> {
        let rd = self.beta_transac::<3>(trans, ld)?;
        if rd == NULL_DART_ID {
            // ?
            Ok(())
        } else {
            self.unlink::<3>(trans, ld)?;

            let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
            let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);

            // faces
            let l_face = l_side.clone().min().expect("E: unreachable");
            let r_face = r_side.clone().min().expect("E: unreachable");
            self.attributes
                .try_split_face_attributes(trans, l_face, r_face, l_face.max(r_face))?;

            for (l, r) in l_side.zip(r_side) {
                // edge
                let (eid_l, eid_r) = (
                    self.edge_id_transac(trans, l)?,
                    self.edge_id_transac(trans, r)?,
                );
                self.attributes
                    .try_split_edge_attributes(trans, eid_l, eid_r, eid_l.max(eid_r))?;
                let b1l = self.beta_transac::<1>(trans, l)?;

                // vertices
                let (vid_l, vid_r) = (
                    if b1l == NULL_DART_ID {
                        let b2l = self.beta_transac::<2>(trans, l)?;
                        if b2l == NULL_DART_ID {
                            NULL_DART_ID // (*)
                        } else {
                            self.vertex_id_transac(trans, b2l)?
                        }
                    } else {
                        self.vertex_id_transac(trans, b1l)?
                    },
                    self.vertex_id_transac(trans, r)?,
                );
                self.vertices
                    .try_split(trans, vid_l, vid_r, vid_l.max(vid_r))?;
                self.attributes.try_split_vertex_attributes(
                    trans,
                    vid_l,
                    vid_r,
                    vid_l.max(vid_r),
                )?;
                if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                    let b1r = self.beta_transac::<1>(trans, r)?;
                    let (lvid_l, lvid_r) = (
                        self.vertex_id_transac(trans, l)?,
                        if b1r == NULL_DART_ID {
                            let b2r = self.beta_transac::<2>(trans, r)?;
                            if b2r == NULL_DART_ID {
                                NULL_DART_ID // (*)
                            } else {
                                self.vertex_id_transac(trans, b2r)?
                            }
                        } else {
                            self.vertex_id_transac(trans, b1r)?
                        },
                    );
                    self.vertices
                        .try_split(trans, lvid_l, lvid_r, lvid_l.max(lvid_r))?;
                    self.attributes.try_split_vertex_attributes(
                        trans,
                        lvid_l,
                        lvid_r,
                        lvid_l.max(lvid_r),
                    )?;
                }
            }
            Ok(())
        }
    }

    /// 3-unsew operation.
    pub(crate) fn force_three_unsew(&self, ld: DartIdType) {
        atomically(|trans| {
            let rd = self.beta_transac::<3>(trans, ld)?;
            if rd == NULL_DART_ID {
                // ?
                Ok(())
            } else {
                self.unlink::<3>(trans, ld)?;

                let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
                let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);

                // faces
                let l_face = l_side.clone().min().expect("E: unreachable");
                let r_face = r_side.clone().min().expect("E: unreachable");
                self.attributes
                    .split_face_attributes(trans, l_face, r_face, l_face.max(r_face))?;

                for (l, r) in l_side.zip(r_side) {
                    // edge
                    let (eid_l, eid_r) = (
                        self.edge_id_transac(trans, l)?,
                        self.edge_id_transac(trans, r)?,
                    );
                    self.attributes
                        .split_edge_attributes(trans, eid_l, eid_r, eid_l.max(eid_r))?;
                    let b1l = self.beta_transac::<1>(trans, l)?;

                    // vertices
                    let (vid_l, vid_r) = (
                        if b1l == NULL_DART_ID {
                            let b2l = self.beta_transac::<2>(trans, l)?;
                            if b2l == NULL_DART_ID {
                                NULL_DART_ID // (*)
                            } else {
                                self.vertex_id_transac(trans, b2l)?
                            }
                        } else {
                            self.vertex_id_transac(trans, b1l)?
                        },
                        self.vertex_id_transac(trans, r)?,
                    );
                    self.vertices.split(trans, vid_l, vid_r, vid_l.max(vid_r))?;
                    self.attributes.split_vertex_attributes(
                        trans,
                        vid_l,
                        vid_r,
                        vid_l.max(vid_r),
                    )?;
                    if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                        let b1r = self.beta_transac::<1>(trans, r)?;
                        let (lvid_l, lvid_r) = (
                            self.vertex_id_transac(trans, l)?,
                            if b1r == NULL_DART_ID {
                                let b2r = self.beta_transac::<2>(trans, r)?;
                                if b2r == NULL_DART_ID {
                                    NULL_DART_ID // (*)
                                } else {
                                    self.vertex_id_transac(trans, b2r)?
                                }
                            } else {
                                self.vertex_id_transac(trans, b1r)?
                            },
                        );
                        self.vertices
                            .split(trans, lvid_l, lvid_r, lvid_l.max(lvid_r))?;
                        self.attributes.split_vertex_attributes(
                            trans,
                            lvid_l,
                            lvid_r,
                            lvid_l.max(lvid_r),
                        )?;
                    }
                }
                Ok(())
            }
        });
    }
}
