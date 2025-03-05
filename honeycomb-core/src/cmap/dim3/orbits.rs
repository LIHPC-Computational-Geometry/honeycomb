//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

use std::collections::{HashSet, VecDeque};

use crate::cmap::{CMap3, DartIdType, NULL_DART_ID, OrbitPolicy};
use crate::geometry::CoordsFloat;

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
    /// [PR]: https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/293
    pub fn orbit(
        &self,
        opolicy: OrbitPolicy,
        dart_id: DartIdType,
    ) -> impl Iterator<Item = DartIdType> {
        let mut pending = VecDeque::new();
        let mut marked: HashSet<DartIdType> = HashSet::new();
        pending.push_back(dart_id);
        marked.insert(NULL_DART_ID);
        marked.insert(dart_id); // we're starting here, so we mark it beforehand

        // FIXME: move the match block out of the iterator
        std::iter::from_fn(move || {
            while let Some(d) = pending.pop_front() {
                // compute the next images
                match opolicy {
                    // B3oB2, B1oB3, B1oB2, B3oB0, B2oB0
                    OrbitPolicy::Vertex => {
                        // b3(b2(d))
                        let image1 = self.beta::<3>(self.beta::<2>(d));
                        if marked.insert(image1) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image1);
                        }
                        // b1(b3(d))
                        let image2 = self.beta::<1>(self.beta::<3>(d));
                        if marked.insert(image2) {
                            pending.push_back(image2);
                        }
                        // b1(b2(d))
                        let image3 = self.beta::<1>(self.beta::<2>(d));
                        if marked.insert(image3) {
                            pending.push_back(image3);
                        }
                        // b3(b0(d))
                        let image4 = self.beta::<3>(self.beta::<0>(d));
                        if marked.insert(image4) {
                            pending.push_back(image4);
                        }
                        // b2(b0(d))
                        let image5 = self.beta::<2>(self.beta::<0>(d));
                        if marked.insert(image5) {
                            pending.push_back(image5);
                        }
                    }
                    // B3oB2, B1oB3, B1oB2
                    OrbitPolicy::VertexLinear => {
                        let image1 = self.beta::<3>(self.beta::<2>(d));
                        if marked.insert(image1) {
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<1>(self.beta::<3>(d));
                        if marked.insert(image2) {
                            pending.push_back(image2);
                        }
                        // b1(b2(d))
                        let image3 = self.beta::<1>(self.beta::<2>(d));
                        if marked.insert(image3) {
                            pending.push_back(image3);
                        }
                    }
                    // B2, B3
                    OrbitPolicy::Edge => {
                        let image1 = self.beta::<2>(d);
                        if marked.insert(image1) {
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<3>(d);
                        if marked.insert(image2) {
                            pending.push_back(image2);
                        }
                    }
                    // B1, B0, B3
                    OrbitPolicy::Face => {
                        let image1 = self.beta::<1>(d);
                        if marked.insert(image1) {
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<0>(d);
                        if marked.insert(image2) {
                            pending.push_back(image2);
                        }
                        let image3 = self.beta::<3>(d);
                        if marked.insert(image3) {
                            pending.push_back(image3);
                        }
                    }
                    // B1, B3
                    OrbitPolicy::FaceLinear => {
                        let image1 = self.beta::<1>(d);
                        if marked.insert(image1) {
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<3>(d);
                        if marked.insert(image2) {
                            pending.push_back(image2);
                        }
                    }
                    // B1, B0, B2
                    OrbitPolicy::Volume => {
                        let image1 = self.beta::<1>(d);
                        if marked.insert(image1) {
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<0>(d);
                        if marked.insert(image2) {
                            pending.push_back(image2);
                        }
                        let image3 = self.beta::<2>(d);
                        if marked.insert(image3) {
                            pending.push_back(image3);
                        }
                    }
                    // B1, B2
                    OrbitPolicy::VolumeLinear => {
                        let image1 = self.beta::<1>(d);
                        if marked.insert(image1) {
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<2>(d);
                        if marked.insert(image2) {
                            pending.push_back(image2);
                        }
                    }
                    OrbitPolicy::Custom(beta_slice) => {
                        for beta_id in beta_slice {
                            let image = self.beta_rt(*beta_id, d);
                            if marked.insert(image) {
                                pending.push_back(image);
                            }
                        }
                    }
                }

                return Some(d);
            }
            None // queue is empty, we're done
        })
    }

    /// Return the orbit defined by a dart and its `I`-cell.
    ///
    /// # Usage
    ///
    /// The [`Orbit3`] can be iterated upon to retrieve all dart members of the cell. Note that
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
