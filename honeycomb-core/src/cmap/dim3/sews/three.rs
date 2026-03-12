//! 3D sew implementations

use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    cmap::{
        CMap3, DartIdType, EdgeIdType, LinkError, NULL_DART_ID, OrbitPolicy, SewError, VertexIdType,
    },
    geometry::CoordsFloat,
    stm::{Transaction, TransactionClosureResult, abort, try_or_coerce},
};

/// **3-(un)sews internals**
impl<T: CoordsFloat> CMap3<T> {
    /// 3-sew operation.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn three_sew_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        // build orbits of each side of the future face
        // the traversal is equivalent to a beta1/beta0 and beta0/beta1 orbit on respective sides,
        // we do it manually to detect asymmetry
        let mut l_side = Vec::with_capacity(10);
        let mut r_side = Vec::with_capacity(10);
        l_side.push(ld);
        r_side.push(rd);
        let (mut l, mut r) = (self.beta_tx::<1>(t, ld)?, self.beta_tx::<0>(t, rd)?);
        let mut open = false;
        // while we haven't looped, or reached an end
        while l != ld && l != NULL_DART_ID {
            if r == NULL_DART_ID {
                // (*)
                abort(SewError::FailedLink(LinkError::AsymmetricalFaces(ld, rd)))?;
            }
            l_side.push(l);
            r_side.push(r);
            (l, r) = (self.beta_tx::<1>(t, l)?, self.beta_tx::<0>(t, r)?);
        }
        if l == NULL_DART_ID {
            open = true;
            // the face was open, so we need to cover the other direction
            // for meshes, we should be working on complete faces at all times,
            // so branch prediction will hopefully save use
            if r != NULL_DART_ID {
                // (*)
                abort(SewError::FailedLink(LinkError::AsymmetricalFaces(ld, rd)))?;
            }
            (l, r) = (self.beta_tx::<0>(t, ld)?, self.beta_tx::<1>(t, rd)?);
            while l != NULL_DART_ID {
                if r == NULL_DART_ID {
                    // (*)
                    abort(SewError::FailedLink(LinkError::AsymmetricalFaces(ld, rd)))?;
                }
                l_side.push(l);
                r_side.push(r);
                (l, r) = (self.beta_tx::<0>(t, l)?, self.beta_tx::<1>(t, r)?);
            }
        }
        // (*): if we land on NULL on one side, the other side should be NULL as well
        //      if that is not the case, it means (either):
        //      - we're trying to sew open faces with a different number of darts
        //      - we're trying to sew open faces that are offset by one (or more) dart(s)
        //      in both case, this is way too clunky to be considered valid

        let l_face = l_side.iter().min().copied().expect("E: unreachable");
        let r_face = r_side.iter().min().copied().expect("E: unreachable");
        let mut edges: Vec<(EdgeIdType, EdgeIdType)> = Vec::with_capacity(10);
        let mut vertices: Vec<(VertexIdType, VertexIdType)> = Vec::with_capacity(10);

        // read edge + vertex on the b1ld side. if b0ld == NULL, we need to read the left vertex
        for (&l, &r) in l_side.iter().zip(r_side.iter()) {
            edges.push((self.edge_id_tx(t, l)?, self.edge_id_tx(t, r)?));
            let (b1l, b2l) = (self.beta_tx::<1>(t, l)?, self.beta_tx::<2>(t, l)?);
            // this monster statement is necessary to handle open faces
            vertices.push((
                self.vertex_id_tx(t, b1l.max(b2l))?,
                self.vertex_id_tx(t, r)?,
            ));
            // one more for good measures (aka open faces)
            if self.beta_tx::<0>(t, l)? == NULL_DART_ID {
                let (b1r, b2r) = (self.beta_tx::<1>(t, r)?, self.beta_tx::<2>(t, r)?);
                vertices.push((
                    self.vertex_id_tx(t, l)?,
                    self.vertex_id_tx(t, b1r.max(b2r))?,
                ));
            }
        }

        // FIXME: we only check orientation of the arg darts
        // ideally, we want to check every sewn pair
        {
            let (vid_l, vid_r, vid_b1l, vid_b1r) = if open {
                let (l, r) = (ld, rd);
                let (b1l, b2l, b1r, b2r) = (
                    self.beta_tx::<1>(t, l)?,
                    self.beta_tx::<2>(t, l)?,
                    self.beta_tx::<1>(t, r)?,
                    self.beta_tx::<2>(t, r)?,
                );
                (
                    self.vertex_id_tx(t, l)?,
                    self.vertex_id_tx(t, r)?,
                    self.vertex_id_tx(t, b1l.max(b2l))?,
                    self.vertex_id_tx(t, b1r.max(b2r))?,
                )
            } else {
                let (vid_b1l, vid_r) = vertices[0];
                let &(vid_l, vid_b1r) = vertices.last().unwrap();
                (vid_l, vid_r, vid_b1l, vid_b1r)
            };

            if let (
                // (lhs/b1rhs) vertices
                Some(l_vertex),
                Some(b1r_vertex),
                // (b1lhs/rhs) vertices
                Some(b1l_vertex),
                Some(r_vertex),
            ) = (
                // (lhs/b1rhs)
                self.vertices.read(t, vid_l)?,
                self.vertices.read(t, vid_b1r)?,
                // (b1lhs/rhs)
                self.vertices.read(t, vid_b1l)?,
                self.vertices.read(t, vid_r)?,
            ) {
                let lhs_vector = b1l_vertex - l_vertex;
                let rhs_vector = b1r_vertex - r_vertex;
                // dot product should be negative if the two darts have opposite direction
                // we could also put restriction on the angle made by the two darts to prevent
                // drastic deformation
                if lhs_vector.dot(&rhs_vector) >= T::zero() {
                    abort(SewError::BadGeometry(3, ld, rd))?;
                }
            }
        }

        // topology update
        for (l, r) in l_side.into_iter().zip(r_side) {
            try_or_coerce!(self.betas.three_link_core(t, l, r), SewError);
        }

        // merge face, edge, vertex attributes
        try_or_coerce!(
            self.attributes.merge_attributes(
                t,
                OrbitPolicy::Face,
                l_face.min(r_face),
                l_face,
                r_face
            ),
            SewError
        );
        for (eid_l, eid_r) in edges.into_iter().filter(|&(eid_l, eid_r)| {
            eid_l != eid_r && eid_l != NULL_DART_ID && eid_r != NULL_DART_ID
        }) {
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Edge,
                    eid_l.min(eid_r),
                    eid_l,
                    eid_r
                ),
                SewError
            );
        }
        for (vid_l, vid_r) in vertices.into_iter().filter(|&(vid_l, vid_r)| {
            vid_l != vid_r && vid_l != NULL_DART_ID && vid_r != NULL_DART_ID
        }) {
            try_or_coerce!(
                self.vertices.merge(t, vid_l.min(vid_r), vid_l, vid_r),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_l.min(vid_r),
                    vid_l,
                    vid_r
                ),
                SewError
            );
        }

        Ok(())
    }

    /// 3-unsew operation.
    pub(crate) fn three_unsew_tx(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_tx::<3>(t, ld)?;
        let mut l_side = Vec::with_capacity(10);
        let mut r_side = Vec::with_capacity(10);
        l_side.push(ld);
        r_side.push(rd);

        try_or_coerce!(self.betas.three_unlink_core(t, ld), SewError);
        let (mut l, mut r) = (self.beta_tx::<1>(t, ld)?, self.beta_tx::<0>(t, rd)?);
        // while we haven't completed the loop, or reached an end
        while l != ld && l != NULL_DART_ID {
            if l != self.beta_tx::<3>(t, r)? {
                // (*); FIXME: add dedicated err ~LinkError::DivergentStructures ?
                abort(SewError::FailedLink(LinkError::AsymmetricalFaces(ld, rd)))?;
            }
            try_or_coerce!(self.betas.three_unlink_core(t, l), SewError);
            l_side.push(l);
            r_side.push(r);
            (l, r) = (self.beta_tx::<1>(t, l)?, self.beta_tx::<0>(t, r)?);
        }
        // the face was open, so we need to cover the other direction
        // for meshes, we should be working on complete faces at all times,
        // so branch prediction will hopefully save use
        if l == NULL_DART_ID {
            if r != NULL_DART_ID {
                // (**)
                abort(SewError::FailedLink(LinkError::AsymmetricalFaces(ld, rd)))?;
            }
            (l, r) = (self.beta_tx::<0>(t, ld)?, self.beta_tx::<1>(t, rd)?);
            while l != NULL_DART_ID {
                if l != self.beta_tx::<3>(t, r)? {
                    // (*); FIXME: add dedicated err ~LinkError::DivergentStructures ?
                    abort(SewError::FailedLink(LinkError::AsymmetricalFaces(ld, rd)))?;
                }
                assert_eq!(l, self.beta_tx::<3>(t, r)?); // (*)
                try_or_coerce!(self.betas.three_unlink_core(t, l), SewError);
                l_side.push(l);
                r_side.push(r);
                (l, r) = (self.beta_tx::<0>(t, l)?, self.beta_tx::<1>(t, r)?);
            }
        }
        // (*) : this can be changed, but the idea here is to ensure we're unlinking the expected
        //       construct
        // (**): if we land on NULL on one side, the other side should be NULL as well

        // faces
        let l_face = l_side.iter().min().copied().expect("E: unreachable");
        let r_face = r_side.iter().min().copied().expect("E: unreachable");
        try_or_coerce!(
            self.attributes.split_attributes(
                t,
                OrbitPolicy::Face,
                l_face,
                r_face,
                l_face.min(r_face)
            ),
            SewError
        );

        for (l, r) in l_side.into_iter().zip(r_side.into_iter()) {
            // edge
            let (eid_l, eid_r) = (self.edge_id_tx(t, l)?, self.edge_id_tx(t, r)?);
            if eid_l != eid_r {
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Edge,
                        eid_l,
                        eid_r,
                        eid_l.min(eid_r)
                    ),
                    SewError
                );
            }

            // vertices
            let (b1l, b2l) = (self.beta_tx::<1>(t, l)?, self.beta_tx::<2>(t, l)?);
            let (vid_l, vid_r) = (
                self.vertex_id_tx(t, b1l.max(b2l))?,
                self.vertex_id_tx(t, r)?,
            );
            if vid_l != vid_r {
                try_or_coerce!(
                    self.vertices.split(t, vid_l, vid_r, vid_l.min(vid_r)),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.split_attributes(
                        t,
                        OrbitPolicy::Vertex,
                        vid_l,
                        vid_r,
                        vid_l.min(vid_r)
                    ),
                    SewError
                );
            }
            if self.beta_tx::<0>(t, l)? == NULL_DART_ID {
                let (b1r, b2r) = (self.beta_tx::<1>(t, r)?, self.beta_tx::<2>(t, r)?);
                let (lvid_l, lvid_r) = (
                    self.vertex_id_tx(t, l)?,
                    self.vertex_id_tx(t, b1r.max(b2r))?,
                );
                if lvid_l != lvid_r {
                    try_or_coerce!(
                        self.vertices.split(t, lvid_l, lvid_r, lvid_l.min(lvid_r)),
                        SewError
                    );
                    try_or_coerce!(
                        self.attributes.split_attributes(
                            t,
                            OrbitPolicy::Vertex,
                            lvid_l,
                            lvid_r,
                            lvid_l.min(lvid_r),
                        ),
                        SewError
                    );
                }
            }
        }
        Ok(())
    }
}
