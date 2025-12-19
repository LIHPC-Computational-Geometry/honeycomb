//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

use std::collections::VecDeque;

use rustc_hash::FxHashSet as HashSet;

use crate::cmap::{CMap3, DartIdType, NULL_DART_ID, OrbitPolicy, try_from_fn};
use crate::geometry::CoordsFloat;
use crate::stm::{StmClosureResult, Transaction};

impl<T: CoordsFloat> CMap3<T> {
    /// Generic orbit implementation.
    ///
    /// # Arguments
    /// - `opolicy: OrbitPolicy` -- Policy used by the orbit for the BFS.
    /// - `dart_id: DartIdentifier` -- Dart of which the structure will compute the orbit.
    ///
    /// # The search algorithm
    ///
    /// The search algorithm used to establish the list of dart included in the orbit is a
    /// [Breadth-First Search algorithm][WIKIBFS]. This means that:
    ///
    /// - we look at the images of the current dart through all beta functions,
    ///   adding those to a queue, before moving on to the next dart.
    /// - we apply the beta functions in their specified order; This guarantees a consistent and
    ///   predictable result.
    ///
    /// # Performance
    ///
    /// Currently, orbits use two dynamically allocated structures for computation: a `VecDeque`,
    /// and a `HashSet`. There is a possibility to use static thread-local instances to avoid
    /// ephemeral allocations, but [it would require a guard mechanism][PR].
    ///
    /// [WIKIBFS]: https://en.wikipedia.org/wiki/Breadth-first_search
    /// [PR]: https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/293
    #[allow(clippy::needless_for_each, clippy::too_many_lines)]
    #[rustfmt::skip]
    pub fn orbit(
        &self,
        opolicy: OrbitPolicy,
        dart_id: DartIdType,
    ) -> impl Iterator<Item = DartIdType> {
        let mut pending = VecDeque::new();
        let mut marked: HashSet<DartIdType> = HashSet::default();
        pending.push_back(dart_id);
        marked.insert(NULL_DART_ID);
        marked.insert(dart_id); // we're starting here, so we mark it beforehand

        // FIXME: move the match block out of the iterator
        std::iter::from_fn(move || {
            if let Some(d) = pending.pop_front() {
                let mut check = |d: DartIdType| {
                    if marked.insert(d) {
                        pending.push_back(d);
                    }
                };
                // compute the next images
                match opolicy {
                    // B3oB2, B1oB3, B1oB2, B3oB0, B2oB0
                    OrbitPolicy::Vertex => {
                        [
                            self.beta::<1>(self.beta::<2>(d)), // b1(b2(d))
                            self.beta::<2>(self.beta::<0>(d)), // b2(b0(d))
                            self.beta::<1>(self.beta::<3>(d)), // b1(b3(d))
                            self.beta::<3>(self.beta::<0>(d)), // b3(b0(d))
                            self.beta::<2>(self.beta::<3>(d)), // b2(b3(d))
                            self.beta::<3>(self.beta::<2>(d)), // b3(b2(d))
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B3oB2, B1oB3, B1oB2
                    OrbitPolicy::VertexLinear => {
                        [
                            self.beta::<1>(self.beta::<2>(d)), // b1(b2(d))
                            self.beta::<1>(self.beta::<3>(d)), // b1(b3(d))
                            self.beta::<2>(self.beta::<3>(d)), // b2(b3(d))
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B2, B3
                    OrbitPolicy::Edge => {
                        [
                            self.beta::<2>(d),
                            self.beta::<3>(d),
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B1, B0, B3
                    OrbitPolicy::Face => {
                        [
                            self.beta::<1>(d),
                            self.beta::<0>(d),
                            self.beta::<3>(d),
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B1, B3
                    OrbitPolicy::FaceLinear => {
                        [
                            self.beta::<1>(d),
                            self.beta::<3>(d),
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B1, B0, B2
                    OrbitPolicy::Volume => {
                        [
                            self.beta::<1>(d),
                            self.beta::<0>(d),
                            self.beta::<2>(d),
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B1, B2
                    OrbitPolicy::VolumeLinear => {
                        [
                            self.beta::<1>(d),
                            self.beta::<2>(d),
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    OrbitPolicy::Custom(beta_slice) => {
                        for beta_id in beta_slice {
                            let im = self.beta_rt(*beta_id, d);
                            check(im);
                        }
                    }
                }

                return Some(d);
            }
            None // queue is empty, we're done
        })
    }

    /// Generic orbit transactional implementation.
    #[allow(clippy::needless_for_each, clippy::too_many_lines)]
    #[rustfmt::skip]
    pub fn orbit_tx(
        &self,
        t: &mut Transaction,
        opolicy: OrbitPolicy,
        dart_id: DartIdType,
    ) -> impl Iterator<Item = StmClosureResult<DartIdType>> {
        let mut pending = VecDeque::new();
        let mut marked: HashSet<DartIdType> = HashSet::default();
        pending.push_back(dart_id);
        marked.insert(NULL_DART_ID);
        marked.insert(dart_id); // we're starting here, so we mark it beforehand

        // FIXME: move the match block out of the iterator
        try_from_fn(move || {
            if let Some(d) = pending.pop_front() {
                let mut check = |d: DartIdType| {
                    if marked.insert(d) {
                        pending.push_back(d);
                    }
                };
                // compute the next images
                match opolicy {
                    // B1oB2, B2oB0, B1oB3, B3oB0, B2oB3, B3oB2
                    OrbitPolicy::Vertex => {
                        let (b0, b2, b3) = (
                            self.beta_tx::<0>(t, d)?,
                            self.beta_tx::<2>(t, d)?,
                            self.beta_tx::<3>(t, d)?,
                        );
                        [
                            self.beta_tx::<1>(t, b2)?, // b1(b2(d))
                            self.beta_tx::<2>(t, b0)?, // b2(b0(d))
                            self.beta_tx::<1>(t, b3)?, // b1(b3(d))
                            self.beta_tx::<3>(t, b0)?, // b3(b0(d))
                            self.beta_tx::<2>(t, b3)?, // b2(b3(d))
                            self.beta_tx::<3>(t, b2)?, // b3(b2(d))
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B2oB3, B1oB3, B1oB2
                    OrbitPolicy::VertexLinear => {
                        let (b2, b3) =
                            (self.beta_tx::<2>(t, d)?, self.beta_tx::<3>(t, d)?);
                        [
                            self.beta_tx::<1>(t, b2)?, // b1(b2(d))
                            self.beta_tx::<1>(t, b3)?, // b1(b3(d))
                            self.beta_tx::<2>(t, b3)?, // b2(b3(d))
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B2, B3
                    OrbitPolicy::Edge => {
                        [self.beta_tx::<2>(t, d)?, self.beta_tx::<3>(t, d)?]
                            .into_iter()
                            .for_each(check);
                    }
                    // B1, B0, B3
                    OrbitPolicy::Face => {
                        [
                            self.beta_tx::<1>(t, d)?,
                            self.beta_tx::<0>(t, d)?,
                            self.beta_tx::<3>(t, d)?,
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B1, B3
                    OrbitPolicy::FaceLinear => {
                        [
                            self.beta_tx::<1>(t, d)?,
                            self.beta_tx::<3>(t, d)?,
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B1, B0, B2
                    OrbitPolicy::Volume => {
                        [
                            self.beta_tx::<1>(t, d)?,
                            self.beta_tx::<0>(t, d)?,
                            self.beta_tx::<2>(t, d)?,
                        ]
                        .into_iter()
                        .for_each(check);
                    }
                    // B1, B2
                    OrbitPolicy::VolumeLinear => {
                        [self.beta_tx::<1>(t, d)?, self.beta_tx::<2>(t, d)?]
                            .into_iter()
                            .for_each(check);
                    }
                    OrbitPolicy::Custom(beta_slice) => {
                        for beta_id in beta_slice {
                            let im = self.beta_rt_tx(t, *beta_id, d)?;
                            check(im);
                        }
                    }
                }

                return Ok(Some(d));
            }
            Ok(None) // queue is empty, we're done
        })
    }

    /// Return the orbit defined by a dart and its `I`-cell.
    ///
    /// # Usage
    ///
    /// The returned item can be iterated upon to retrieve all dart members of the cell. Note that
    /// **the dart passed as an argument is included as the first element of the returned orbit**.
    ///
    /// # Panics
    ///
    /// The method will panic if *I* is not 0, 1, 2, or 3.
    #[must_use = "unused return value"]
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdType) -> impl Iterator<Item = DartIdType> {
        assert!(I < 4);
        match I {
            0 => self.orbit(OrbitPolicy::Vertex, dart_id),
            1 => self.orbit(OrbitPolicy::Edge, dart_id),
            2 => self.orbit(OrbitPolicy::Face, dart_id),
            3 => self.orbit(OrbitPolicy::Volume, dart_id),
            _ => unreachable!(),
        }
    }
}
