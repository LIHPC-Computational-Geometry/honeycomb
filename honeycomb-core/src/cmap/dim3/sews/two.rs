//! 2D sew implementations

use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    cmap::{CMap3, DartIdType, NULL_DART_ID, OrbitPolicy, SewError, VertexIdType},
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
        let b1ld = self.beta_tx::<1>(t, ld)?.max(self.beta_tx::<3>(t, ld)?);
        let b1rd = self.beta_tx::<1>(t, rd)?.max(self.beta_tx::<3>(t, rd)?);

        // fetch cell IDs before link
        let eid_l = self.edge_id_tx(t, ld)?;
        let eid_r = self.edge_id_tx(t, rd)?;
        // (lhs/b1rhs) vertex
        let (vid_l, vid_b1r, new_orbit_lb1r) = {
            let mut new_orbit = Vec::with_capacity(16);
            let mut vid_l = ld;
            let mut vid_r = b1rd;
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, ld) {
                let d = d?;
                new_orbit.push(d);
                if d < vid_l {
                    vid_l = d;
                }
            }
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, b1rd) {
                let d = d?;
                new_orbit.push(d);
                if d < vid_r {
                    vid_r = d;
                }
            }
            (vid_l, vid_r, new_orbit)
        };
        // (b1lhs/rhs) vertex
        let (vid_b1l, vid_r, new_orbit_b1lr) = {
            let mut new_orbit = Vec::with_capacity(16);
            let mut vid_l = b1ld as VertexIdType;
            let mut vid_r = rd as VertexIdType;
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, b1ld) {
                let d = d?;
                new_orbit.push(d);
                if d < vid_l {
                    vid_l = d as VertexIdType;
                }
            }
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, rd) {
                let d = d?;
                new_orbit.push(d);
                if d < vid_r {
                    vid_r = d as VertexIdType;
                }
            }
            (vid_l, vid_r, new_orbit)
        };

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
                abort(SewError::BadGeometry(2, ld, rd))?;
            }
        }

        try_or_coerce!(self.two_link(t, ld, rd), SewError);

        // merge edge attributes
        try_or_coerce!(
            self.attributes
                .merge_attributes(t, OrbitPolicy::Edge, eid_l.min(eid_r), eid_l, eid_r),
            SewError
        );

        // merge vertex attributes depending on whether
        // - there was an existing orbit at each end
        // - there was an existing orbit on each side
        if b1rd != NULL_DART_ID && vid_l != vid_b1r {
            let new_vid = vid_l.min(vid_b1r);
            try_or_coerce!(self.vertices.merge(t, new_vid, vid_l, vid_b1r), SewError);
            try_or_coerce!(
                self.attributes
                    .merge_attributes(t, OrbitPolicy::Vertex, new_vid, vid_l, vid_b1r),
                SewError
            );
            if let Some(ref vids) = self.vertex_ids {
                for d in new_orbit_lb1r {
                    vids[d as usize].write(t, new_vid)?;
                }
            }
        }
        if b1ld != NULL_DART_ID && vid_b1l != vid_r {
            let new_vid = vid_b1l.min(vid_r);
            try_or_coerce!(self.vertices.merge(t, new_vid, vid_b1l, vid_r), SewError);
            try_or_coerce!(
                self.attributes
                    .merge_attributes(t, OrbitPolicy::Vertex, new_vid, vid_b1l, vid_r),
                SewError
            );
            if let Some(ref vids) = self.vertex_ids {
                for d in new_orbit_b1lr {
                    vids[d as usize].write(t, new_vid)?;
                }
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
        let rd = self.beta_tx::<2>(t, ld)?;
        let b1ld = self.beta_tx::<1>(t, ld)?.max(self.beta_tx::<3>(t, ld)?);
        let b1rd = self.beta_tx::<1>(t, rd)?.max(self.beta_tx::<3>(t, rd)?);

        try_or_coerce!(self.two_unlink(t, ld), SewError);

        let (eid_newl, eid_newr) = (self.edge_id_tx(t, ld)?, self.edge_id_tx(t, rd)?);
        let (vid_l_newl, vid_l_newr, new_l_orbit_l, new_l_orbit_r) = {
            let mut new_l_orbit = Vec::with_capacity(16);
            let mut vid_l_new = ld as VertexIdType;
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, ld) {
                let d = d?;
                new_l_orbit.push(d);
                if d < vid_l_new {
                    vid_l_new = d as VertexIdType;
                }
            }
            let mut new_r_orbit = Vec::with_capacity(16);
            let mut vid_r_new = b1rd as VertexIdType;
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, b1rd) {
                let d = d?;
                new_r_orbit.push(d);
                if d < vid_r_new {
                    vid_r_new = d as VertexIdType;
                }
            }
            (vid_l_new, vid_r_new, new_l_orbit, new_r_orbit)
        };
        let (vid_r_newl, vid_r_newr, new_r_orbit_l, new_r_orbit_r) = {
            let mut new_l_orbit = Vec::with_capacity(16);
            let mut vid_l_new = b1ld as VertexIdType;
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, b1ld) {
                let d = d?;
                new_l_orbit.push(d);
                if d < vid_l_new {
                    vid_l_new = d as VertexIdType;
                }
            }
            let mut new_r_orbit = Vec::with_capacity(16);
            let mut vid_r_new = rd as VertexIdType;
            for d in self.orbit_tx(t, OrbitPolicy::Vertex, rd) {
                let d = d?;
                new_r_orbit.push(d);
                if d < vid_r_new {
                    vid_r_new = d as VertexIdType;
                }
            }
            (vid_l_new, vid_r_new, new_l_orbit, new_r_orbit)
        };

        // split edge attributes
        try_or_coerce!(
            self.attributes.split_attributes(
                t,
                OrbitPolicy::Edge,
                eid_newl,
                eid_newr,
                eid_newl.min(eid_newr),
            ),
            SewError
        );

        // split vertex attributes depending on whether two distinct orbits were formed by unlink
        if b1rd != NULL_DART_ID && vid_l_newl != vid_l_newr {
            try_or_coerce!(
                self.vertices
                    .split(t, vid_l_newl, vid_l_newr, vid_l_newl.min(vid_l_newr)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_l_newl,
                    vid_l_newr,
                    vid_l_newl.min(vid_l_newr)
                ),
                SewError
            );
            if let Some(ref vids) = self.vertex_ids {
                for d in new_l_orbit_l {
                    vids[d as usize].write(t, vid_l_newl)?;
                }
                for d in new_l_orbit_r {
                    vids[d as usize].write(t, vid_l_newr)?;
                }
            }
        }
        if b1ld != NULL_DART_ID && vid_r_newl != vid_r_newr {
            try_or_coerce!(
                self.vertices
                    .split(t, vid_r_newl, vid_r_newr, vid_r_newl.min(vid_r_newr)),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_r_newl,
                    vid_r_newr,
                    vid_r_newl.min(vid_r_newr)
                ),
                SewError
            );
            if let Some(ref vids) = self.vertex_ids {
                for d in new_r_orbit_l {
                    vids[d as usize].write(t, vid_r_newl)?;
                }
                for d in new_r_orbit_r {
                    vids[d as usize].write(t, vid_r_newr)?;
                }
            }
        }

        Ok(())
    }
}
