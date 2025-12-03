//! 2D link implementations

use crate::cmap::{CMap3, DartIdType, LinkError, NULL_DART_ID, OrbitPolicy};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, atomically_with_err};

/// 2-links
impl<T: CoordsFloat> CMap3<T> {
    /// 2-link operation.
    pub(crate) fn two_link(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.two_link_core(t, ld, rd)?;
        if let Some(ref vids) = self.vid_cache {
            // first vertex
            let b1l = self.beta_tx::<1>(t, ld)?;
            let b3l = self.beta_tx::<3>(t, ld)?;
            let d = b1l.max(b3l);
            // one or both is not zero -> two orbits were merged
            if d != NULL_DART_ID {
                let ll = ld;
                let rr = d;
                let lvid = vids[ll as usize].read(t)?;
                let rvid = vids[rr as usize].read(t)?;
                if lvid != rvid {
                    let new_vid = lvid.min(rvid);
                    let mut darts = Vec::with_capacity(16);
                    for d in self.orbit_tx(t, OrbitPolicy::Vertex, ll) {
                        darts.push(d?);
                    }
                    for d in darts {
                        vids[d as usize].write(t, new_vid)?;
                    }
                }
            }
            // second vertex
            let b1r = self.beta_tx::<1>(t, rd)?;
            let b3r = self.beta_tx::<3>(t, rd)?;
            let d = b1r.max(b3r);
            if d != NULL_DART_ID {
                let ll = rd;
                let rr = d;
                let lvid = vids[ll as usize].read(t)?;
                let rvid = vids[rr as usize].read(t)?;
                if lvid != rvid {
                    let new_vid = lvid.min(rvid);
                    let mut darts = Vec::with_capacity(16);
                    for d in self.orbit_tx(t, OrbitPolicy::Vertex, ll) {
                        darts.push(d?);
                    }
                    for d in darts {
                        vids[d as usize].write(t, new_vid)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 2-link operation.
    pub(crate) fn force_two_link(
        &self,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), LinkError> {
        atomically_with_err(|t| self.betas.two_link_core(t, lhs_dart_id, rhs_dart_id))
    }
}

/// 2-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 2-unlink operation.
    pub(crate) fn two_unlink(
        &self,
        t: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.two_unlink_core(t, lhs_dart_id)
    }

    /// 2-unlink operation.
    pub(crate) fn force_two_unlink(&self, lhs_dart_id: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|t| self.betas.two_unlink_core(t, lhs_dart_id))
    }
}
