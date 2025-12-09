//! 3D link implementations

use crate::cmap::{CMap3, DartIdType, LinkError, NULL_DART_ID, OrbitPolicy};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, abort, atomically_with_err};

/// 3-links
impl<T: CoordsFloat> CMap3<T> {
    /// 3-link operation.
    pub(crate) fn three_link(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        self.betas.three_link_core(t, ld, rd)?;
        let mut pairs = Vec::with_capacity(16);
        let (mut lside, mut rside) = (self.beta_tx::<1>(t, ld)?, self.beta_tx::<0>(t, rd)?);
        pairs.push((lside.max(self.beta_tx::<2>(t, ld)?), rd));
        // while we haven't completed the loop, or reached an end
        while lside != ld && lside != NULL_DART_ID {
            let (b1l, b2l) = (self.beta_tx::<1>(t, lside)?, self.beta_tx::<2>(t, lside)?);
            pairs.push((b1l.max(b2l), rside));
            if rside == NULL_DART_ID {
                // (*)
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            self.betas.three_link_core(t, lside, rside)?;
            (lside, rside) = (b1l, self.beta_tx::<0>(t, rside)?);
        }
        // the face was open, so we need to cover the other direction
        // for meshes, we should be working on complete faces at all times,
        // so branch prediction will hopefully save use
        if lside == NULL_DART_ID {
            if rside != NULL_DART_ID {
                // (*)
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            (lside, rside) = (self.beta_tx::<0>(t, ld)?, self.beta_tx::<1>(t, rd)?);
            while lside != NULL_DART_ID {
                let (b1l, b2l) = (self.beta_tx::<1>(t, lside)?, self.beta_tx::<2>(t, lside)?);
                pairs.push((b1l.max(b2l), rside));
                if rside == NULL_DART_ID {
                    // (*)
                    abort(LinkError::AsymmetricalFaces(ld, rd))?;
                }
                self.betas.three_link_core(t, lside, rside)?;
                let b1r = self.beta_tx::<1>(t, rside)?;
                if b1r == NULL_DART_ID {
                    let b2r = self.beta_tx::<2>(t, rside)?;
                    if b2r != NULL_DART_ID {
                        pairs.push((lside, b2r));
                    }
                }
                (lside, rside) = (self.beta_tx::<0>(t, lside)?, b1r);
            }
        }
        // (*): if we land on NULL on one side, the other side should be NULL as well
        //      if that is not the case, it means (either):
        //      - we're trying to sew open faces with a different number of darts
        //      - we're trying to sew open faces that are offset by one (or more) dart(s)
        //      in both case, this is way too clunky to be considered valid

        if let Some(ref vids) = self.vid_cache {
            for (ll, rr) in pairs {
                if ll == NULL_DART_ID || rr == NULL_DART_ID {
                    continue;
                }
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

    /// 3-link operation.
    pub(crate) fn force_three_link(&self, ld: DartIdType, rd: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|t| self.three_link(t, ld, rd))
    }
}

/// 3-unlinks
impl<T: CoordsFloat> CMap3<T> {
    /// 3-unlink operation.
    pub(crate) fn three_unlink(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        let rd = self.beta_tx::<3>(t, ld)?;

        self.betas.three_unlink_core(t, ld)?;
        let mut pairs = Vec::with_capacity(16);
        let (mut lside, mut rside) = (self.beta_tx::<1>(t, ld)?, self.beta_tx::<0>(t, rd)?);
        pairs.push((lside.max(self.beta_tx::<2>(t, ld)?), rd));
        // while we haven't completed the loop, or reached an end
        while lside != ld && lside != NULL_DART_ID {
            let (b1l, b2l) = (self.beta_tx::<1>(t, lside)?, self.beta_tx::<2>(t, lside)?);
            pairs.push((b1l.max(b2l), rside));
            if lside != self.beta_tx::<3>(t, rside)? {
                // (*); FIXME: add dedicated err ~LinkError::DivergentStructures ?
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            self.betas.three_unlink_core(t, lside)?;
            (lside, rside) = (b1l, self.beta_tx::<0>(t, rside)?);
        }
        // the face was open, so we need to cover the other direction
        // for meshes, we should be working on complete faces at all times,
        // so branch prediction will hopefully save use
        if lside == NULL_DART_ID {
            if rside != NULL_DART_ID {
                // (**)
                abort(LinkError::AsymmetricalFaces(ld, rd))?;
            }
            (lside, rside) = (self.beta_tx::<0>(t, ld)?, self.beta_tx::<1>(t, rd)?);
            while lside != NULL_DART_ID {
                let (b1l, b2l) = (self.beta_tx::<1>(t, lside)?, self.beta_tx::<2>(t, lside)?);
                pairs.push((b1l.max(b2l), rside));
                if lside != self.beta_tx::<3>(t, rside)? {
                    // (*); FIXME: add dedicated err ~LinkError::DivergentStructures ?
                    abort(LinkError::AsymmetricalFaces(ld, rd))?;
                }
                assert_eq!(lside, self.beta_tx::<3>(t, rside)?); // (*)
                self.betas.three_unlink_core(t, lside)?;
                let b1r = self.beta_tx::<1>(t, rside)?;
                if b1r == NULL_DART_ID {
                    let b2r = self.beta_tx::<2>(t, rside)?;
                    if b2r != NULL_DART_ID {
                        pairs.push((lside, b2r));
                    }
                }
                (lside, rside) = (self.beta_tx::<0>(t, lside)?, self.beta_tx::<1>(t, rside)?);
            }
        }
        // (*) : this can be changed, but the idea here is to ensure we're unlinking the expected
        //       construct
        // (**): if we land on NULL on one side, the other side should be NULL as well

        if let Some(ref vids) = self.vid_cache {
            for (ll, rr) in pairs {
                if ll == NULL_DART_ID || rr == NULL_DART_ID {
                    continue;
                }
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
                let mut rvid = rr;
                for d in self.orbit_tx(t, OrbitPolicy::Vertex, rr) {
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

    /// 3-unlink operation.
    pub(crate) fn force_three_unlink(&self, ld: DartIdType) -> Result<(), LinkError> {
        atomically_with_err(|t| self.three_unlink(t, ld))
    }
}
