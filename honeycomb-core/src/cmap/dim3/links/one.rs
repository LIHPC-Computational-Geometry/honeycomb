//! 1D link implementations

use crate::cmap::{CMap3, DartIdType, LinkError, NULL_DART_ID, OrbitPolicy};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, abort, atomically_with_err};

/// 1-links
impl<T: CoordsFloat> CMap3<T> {
    /// 1-link operation.
    pub(crate) fn one_link(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.one_link_core(t, ld, rd)?;
        let (b3_ld, b3_rd) = (self.beta_tx::<3>(t, ld)?, self.beta_tx::<3>(t, rd)?);
        if b3_ld != NULL_DART_ID && b3_rd != NULL_DART_ID {
            self.betas.one_link_core(t, b3_rd, b3_ld)?;
        }
        if let Some(ref vids) = self.vid_cache {
            let b2_ld = self.beta_tx::<2>(t, ld)?;
            let ll = b2_ld.max(b3_ld);
            if ll != NULL_DART_ID {
                let lvid = vids[ll as usize].read(t)?;
                let rvid = vids[rd as usize].read(t)?;
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

    /// 1-link operation.
    ///
    /// This variant is equivalent to `one_link`, but internally uses a transaction that will be
    /// retried until validated.
    pub(crate) fn force_one_link(&self, ld: DartIdType, rd: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|t| self.one_link(t, ld, rd))
    }
}

/// 1-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 1-unlink operation.
    pub(crate) fn one_unlink(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        let rd = self.beta_tx::<1>(t, ld)?;
        self.betas.one_unlink_core(t, ld)?;
        let (b3_ld, b3_rd) = (self.beta_tx::<3>(t, ld)?, self.beta_tx::<3>(t, rd)?);
        if b3_ld != NULL_DART_ID && b3_rd != NULL_DART_ID {
            if self.beta_tx::<1>(t, b3_rd)? != b3_ld {
                // FIXME: add dedicated variant ~LinkError::DivergentStructures ?
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            self.betas.one_unlink_core(t, b3_rd)?;
        }
        if let Some(ref vids) = self.vid_cache {
            let b2_ld = self.beta_tx::<2>(t, ld)?;
            let ll = b2_ld.max(b3_ld);
            if ll != NULL_DART_ID {
                let mut l_orbit = Vec::with_capacity(16);
                let mut lvid = ll;
                for d in self.orbit_tx(t, OrbitPolicy::Vertex, ll) {
                    let d = d?;
                    l_orbit.push(d);
                    if d < lvid {
                        lvid = d;
                    }
                }
                let mut r_orbit = Vec::with_capacity(16);
                let mut rvid = rd;
                for d in self.orbit_tx(t, OrbitPolicy::Vertex, rd) {
                    let d = d?;
                    r_orbit.push(d);
                    if d < rvid {
                        rvid = d;
                    }
                }
                if lvid != rvid {
                    for d in l_orbit {
                        vids[d as usize].write(t, lvid)?;
                    }
                    for d in r_orbit {
                        vids[d as usize].write(t, rvid)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 1-unlink operation.
    ///
    /// This variant is equivalent to `one_unlink`, but internally uses a transaction that will be
    /// retried until validated.
    pub(crate) fn force_one_unlink(&self, ld: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|t| self.one_unlink(t, ld))
    }
}
