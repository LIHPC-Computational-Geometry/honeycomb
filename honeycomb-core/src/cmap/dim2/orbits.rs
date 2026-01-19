//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

use std::collections::VecDeque;

use rustc_hash::FxHashSet as HashSet;

use crate::cmap::{CMap2, DartIdType, NULL_DART_ID, OrbitPolicy, try_from_fn};
use crate::geometry::CoordsFloat;
use crate::stm::{StmClosureResult, Transaction};

/// **Orbits**
impl<T: CoordsFloat> CMap2<T> {
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
    #[allow(clippy::needless_for_each)]
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
                // I have to define the closure here due to mutability constraints
                let mut check = |d: DartIdType| {
                    if marked.insert(d) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        pending.push_back(d);
                    }
                };
                // compute the next images
                match opolicy {
                    OrbitPolicy::Vertex => {
                        let im1 = self.beta::<1>(self.beta::<2>(d));
                        let im2 = self.beta::<2>(self.beta::<0>(d));
                        check(im1);
                        check(im2);
                    }
                    OrbitPolicy::VertexLinear => {
                        let im = self.beta::<1>(self.beta::<2>(d));
                        check(im);
                    }
                    OrbitPolicy::Edge => {
                        let im = self.beta::<2>(d);
                        check(im);
                    }
                    OrbitPolicy::Face => {
                        let im1 = self.beta::<1>(d);
                        let im2 = self.beta::<0>(d);
                        check(im1);
                        check(im2);
                    }
                    OrbitPolicy::FaceLinear => {
                        let im = self.beta::<1>(d);
                        check(im);
                    }
                    OrbitPolicy::Custom(beta_slice) => {
                        for beta_id in beta_slice {
                            let im = self.beta_rt(*beta_id, d);
                            check(im);
                        }
                    }
                    OrbitPolicy::Volume | OrbitPolicy::VolumeLinear => {
                        unimplemented!("3-cells aren't defined for 2-maps")
                    }
                }

                return Some(d);
            }
            None // queue is empty, we're done
        })
    }

    /// Generic orbit transactional implementation.
    #[allow(clippy::needless_for_each)]
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

        try_from_fn(move || {
            if let Some(d) = pending.pop_front() {
                // I have to define the closure here due to mutability constraints
                let mut check = |d: DartIdType| {
                    if marked.insert(d) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        pending.push_back(d);
                    }
                };
                match opolicy {
                    OrbitPolicy::Vertex => {
                        let b2 = self.beta_tx::<2>(t, d)?;
                        let b0 = self.beta_tx::<0>(t, d)?;
                        let im1 = self.beta_tx::<1>(t, b2)?;
                        let im2 = self.beta_tx::<2>(t, b0)?;
                        check(im1);
                        check(im2);
                    }
                    OrbitPolicy::VertexLinear => {
                        let b2 = self.beta_tx::<2>(t, d)?;
                        let im = self.beta_tx::<1>(t, b2)?;
                        check(im);
                    }
                    OrbitPolicy::Edge => {
                        let im = self.beta_tx::<2>(t, d)?;
                        check(im);
                    }
                    OrbitPolicy::Face => {
                        let im1 = self.beta_tx::<1>(t, d)?;
                        let im2 = self.beta_tx::<0>(t, d)?;
                        check(im1);
                        check(im2);
                    }
                    OrbitPolicy::FaceLinear => {
                        let im = self.beta_tx::<1>(t, d)?;
                        check(im);
                    }
                    OrbitPolicy::Custom(beta_slice) => {
                        for beta_id in beta_slice {
                            let im = self.beta_rt_tx(t, *beta_id, d)?;
                            check(im);
                        }
                    }
                    OrbitPolicy::Volume | OrbitPolicy::VolumeLinear => {
                        unimplemented!("3-cells aren't defined for 2-maps")
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
    /// The returned item can be iterated upon to retrieve all dart member of the cell. Note that
    /// **the dart passed as an argument is included as the first element of the returned orbit**.
    ///
    /// # Panics
    ///
    /// The method will panic if *I* is not 0, 1 or 2.
    #[must_use = "unused return value"]
    pub fn i_cell<const I: u8>(&self, dart_id: DartIdType) -> impl Iterator<Item = DartIdType> {
        assert!(I < 3);
        match I {
            0 => self.orbit(OrbitPolicy::Vertex, dart_id),
            1 => self.orbit(OrbitPolicy::Edge, dart_id),
            2 => self.orbit(OrbitPolicy::Face, dart_id),
            _ => unreachable!(),
        }
    }
}
