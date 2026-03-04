//! 3D sew implementations

use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    cmap::{CMap3, DartIdType, EdgeIdType, NULL_DART_ID, OrbitPolicy, SewError, VertexIdType},
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
        // using these custom orbits we can get both darts of all sides correctly ordered for merges
        let mut l_side = Vec::with_capacity(10);
        for d in self.orbit_tx(t, OrbitPolicy::Custom(&[1, 0]), ld) {
            l_side.push(d?);
        }
        let mut r_side = Vec::with_capacity(10);
        for d in self.orbit_tx(t, OrbitPolicy::Custom(&[0, 1]), rd) {
            r_side.push(d?);
        }
        let l_face = l_side.iter().min().copied().expect("E: unreachable");
        let r_face = r_side.iter().min().copied().expect("E: unreachable");
        let mut edges: Vec<(EdgeIdType, EdgeIdType)> = Vec::with_capacity(10);
        let mut vertices: Vec<(VertexIdType, VertexIdType)> = Vec::with_capacity(10);

        // read edge + vertex on the b1ld side. if b0ld == NULL, we need to read the left vertex
        for (l, r) in l_side.into_iter().zip(r_side.into_iter()) {
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
            let (l, r) = (ld, rd);
            let (b1l, b2l, b1r, b2r) = (
                self.beta_tx::<1>(t, l)?,
                self.beta_tx::<2>(t, l)?,
                self.beta_tx::<1>(t, r)?,
                self.beta_tx::<2>(t, r)?,
            );
            let (vid_l, vid_r, vid_b1l, vid_b1r) = (
                self.vertex_id_tx(t, l)?,
                self.vertex_id_tx(t, r)?,
                self.vertex_id_tx(t, if b1l == NULL_DART_ID { b2l } else { b1l })?,
                self.vertex_id_tx(t, if b1r == NULL_DART_ID { b2r } else { b1r })?,
            );

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

        // (*): these branch corresponds to incomplete merges (at best),
        //      or incorrect structure (at worst). that's not a problem
        //      because `three_link` will detect inconsistencies
        try_or_coerce!(self.three_link_tx(t, ld, rd), SewError);

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

        try_or_coerce!(self.three_unlink_tx(t, ld), SewError);

        let mut l_side = Vec::with_capacity(10);
        for d in self.orbit_tx(t, OrbitPolicy::Custom(&[1, 0]), ld) {
            l_side.push(d?);
        }
        let mut r_side = Vec::with_capacity(10);
        for d in self.orbit_tx(t, OrbitPolicy::Custom(&[0, 1]), rd) {
            r_side.push(d?);
        }

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

            // vertices
            let (b1l, b2l) = (self.beta_tx::<1>(t, l)?, self.beta_tx::<2>(t, l)?);
            let (vid_l, vid_r) = (
                self.vertex_id_tx(t, b1l.max(b2l))?,
                self.vertex_id_tx(t, r)?,
            );
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
            if self.beta_tx::<0>(t, l)? == NULL_DART_ID {
                let (b1r, b2r) = (self.beta_tx::<1>(t, r)?, self.beta_tx::<2>(t, r)?);
                let (lvid_l, lvid_r) = (
                    self.vertex_id_tx(t, l)?,
                    self.vertex_id_tx(t, b1r.max(b2r))?,
                );
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
        Ok(())
    }
}
