//! 1D sew implementations

use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{CMap3, DartIdType, NULL_DART_ID, OrbitPolicy, SewError, VertexIdType},
    geometry::CoordsFloat,
    stm::{Transaction, TransactionClosureResult, try_or_coerce},
};

#[doc(hidden)]
/// **1-(un)sews internals)**
impl<T: CoordsFloat> CMap3<T> {
    /// 1-sew transactional operation.
    pub(crate) fn one_sew(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b3ld = self.beta_tx::<3>(t, ld)?.max(self.beta_tx::<2>(t, ld)?);

        // manually going through orbits reduces the number of graph traversal to the minimum
        let mut new_orbit = Vec::with_capacity(16);
        let mut vid_l_old = b3ld;
        for d in self.orbit_tx(t, OrbitPolicy::Vertex, b3ld) {
            let d = d?;
            new_orbit.push(d);
            if d < vid_l_old {
                vid_l_old = d as VertexIdType;
            }
        }
        let mut vid_r_old = rd as VertexIdType;
        for d in self.orbit_tx(t, OrbitPolicy::Vertex, rd) {
            let d = d?;
            new_orbit.push(d);
            if d < vid_r_old {
                vid_r_old = d as VertexIdType;
            }
        }

        try_or_coerce!(self.one_link(t, ld, rd), SewError);

        if b3ld != NULL_DART_ID && vid_l_old != vid_r_old {
            let new_vid = vid_r_old.min(vid_l_old);
            try_or_coerce!(
                self.vertices.merge(t, new_vid, vid_l_old, vid_r_old),
                SewError
            );
            try_or_coerce!(
                self.attributes.merge_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    new_vid,
                    vid_l_old,
                    vid_r_old
                ),
                SewError
            );
            if let Some(ref vids) = self.vertex_ids {
                for d in new_orbit {
                    vids[d as usize].write(t, new_vid)?;
                }
            }
        }
        Ok(())
    }

    /// 1-unsew transactional operation.
    pub(crate) fn one_unsew(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_tx::<1>(t, ld)?;
        let b3ld = self.beta_tx::<3>(t, ld)?.max(self.beta_tx::<2>(t, ld)?);

        try_or_coerce!(self.one_unlink(t, ld), SewError);

        let mut new_l_orbit = Vec::with_capacity(16);
        let mut vid_l_new = b3ld;
        for d in self.orbit_tx(t, OrbitPolicy::Vertex, b3ld) {
            let d = d?;
            new_l_orbit.push(d);
            if d < vid_l_new {
                vid_l_new = d as VertexIdType;
            }
        }
        let mut new_r_orbit = Vec::with_capacity(16);
        let mut vid_r_new = rd;
        for d in self.orbit_tx(t, OrbitPolicy::Vertex, rd) {
            let d = d?;
            new_r_orbit.push(d);
            if d < vid_r_new {
                vid_r_new = d as VertexIdType;
            }
        }

        if b3ld != NULL_DART_ID && vid_l_new != vid_r_new {
            let old_vid = vid_l_new.min(vid_r_new);
            try_or_coerce!(
                self.vertices.split(t, vid_l_new, vid_r_new, old_vid),
                SewError
            );
            try_or_coerce!(
                self.attributes.split_attributes(
                    t,
                    OrbitPolicy::Vertex,
                    vid_l_new,
                    vid_r_new,
                    old_vid,
                ),
                SewError
            );
            if let Some(ref vids) = self.vertex_ids {
                for d in new_l_orbit {
                    vids[d as usize].write(t, vid_l_new)?;
                }
                for d in new_r_orbit {
                    vids[d as usize].write(t, vid_r_new)?;
                }
            }
        }
        Ok(())
    }
}
