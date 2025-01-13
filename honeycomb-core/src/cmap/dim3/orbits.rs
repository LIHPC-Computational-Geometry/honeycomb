//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

use crate::cmap::CMap3;
use crate::geometry::CoordsFloat;
use crate::prelude::{DartIdType, OrbitPolicy, NULL_DART_ID};

use std::collections::{HashSet, VecDeque};

#[derive(Clone)]
pub struct Orbit3<'a, T: CoordsFloat> {
    /// Reference to the map containing the beta functions used in the BFS.
    map_handle: &'a CMap3<T>,
    /// Policy used by the orbit for the BFS. It can be predetermined or custom.
    orbit_policy: OrbitPolicy,
    /// Set used to identify which dart is marked during the BFS.
    marked: HashSet<DartIdType>,
    /// Queue used to store which dart must be visited next during the BFS.
    pending: VecDeque<DartIdType>,
}

impl<'a, T: CoordsFloat> Orbit3<'a, T> {
    #[must_use = "unused return value"]
    pub fn new(map_handle: &'a CMap3<T>, orbit_policy: OrbitPolicy, dart: DartIdType) -> Self {
        let mut marked = HashSet::<DartIdType>::new();
        marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit
        marked.insert(dart); // we're starting here, so we mark it beforehand
        let pending = VecDeque::from([dart]);

        Self {
            map_handle,
            orbit_policy,
            marked,
            pending,
        }
    }
}

impl<T: CoordsFloat> Iterator for Orbit3<'_, T> {
    type Item = DartIdType;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(d) = self.pending.pop_front() {
            match self.orbit_policy {
                // B3oB2, B1oB3, B1oB2, B3oB0, B2oB0
                OrbitPolicy::Vertex => {
                    // b3(b2(d))
                    let image1 = self.map_handle.beta::<3>(self.map_handle.beta::<2>(d));
                    if self.marked.insert(image1) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image1);
                    }
                    // b1(b3(d))
                    let image2 = self.map_handle.beta::<1>(self.map_handle.beta::<3>(d));
                    if self.marked.insert(image2) {
                        self.pending.push_back(image2);
                    }
                    // b1(b2(d))
                    let image3 = self.map_handle.beta::<1>(self.map_handle.beta::<2>(d));
                    if self.marked.insert(image3) {
                        self.pending.push_back(image3);
                    }
                    // b3(b0(d))
                    let image4 = self.map_handle.beta::<3>(self.map_handle.beta::<0>(d));
                    if self.marked.insert(image4) {
                        self.pending.push_back(image4);
                    }
                    // b2(b0(d))
                    let image5 = self.map_handle.beta::<2>(self.map_handle.beta::<0>(d));
                    if self.marked.insert(image5) {
                        self.pending.push_back(image5);
                    }
                }
                // B3oB2, B1oB3, B1oB2
                OrbitPolicy::VertexLinear => {
                    let image1 = self.map_handle.beta::<3>(self.map_handle.beta::<2>(d));
                    if self.marked.insert(image1) {
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<1>(self.map_handle.beta::<3>(d));
                    if self.marked.insert(image2) {
                        self.pending.push_back(image2);
                    }
                    // b1(b2(d))
                    let image3 = self.map_handle.beta::<1>(self.map_handle.beta::<2>(d));
                    if self.marked.insert(image3) {
                        self.pending.push_back(image3);
                    }
                }
                // B2, B3
                OrbitPolicy::Edge => {
                    let image1 = self.map_handle.beta::<2>(d);
                    if self.marked.insert(image1) {
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<3>(d);
                    if self.marked.insert(image2) {
                        self.pending.push_back(image2);
                    }
                }
                // B1, B0, B3
                OrbitPolicy::Face => {
                    let image1 = self.map_handle.beta::<1>(d);
                    if self.marked.insert(image1) {
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<0>(d);
                    if self.marked.insert(image2) {
                        self.pending.push_back(image2);
                    }
                    let image3 = self.map_handle.beta::<3>(d);
                    if self.marked.insert(image3) {
                        self.pending.push_back(image3);
                    }
                }
                // B1, B3
                OrbitPolicy::FaceLinear => {
                    let image1 = self.map_handle.beta::<1>(d);
                    if self.marked.insert(image1) {
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<3>(d);
                    if self.marked.insert(image2) {
                        self.pending.push_back(image2);
                    }
                }
                // B1, B0, B2
                OrbitPolicy::Volume => {
                    let image1 = self.map_handle.beta::<1>(d);
                    if self.marked.insert(image1) {
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<0>(d);
                    if self.marked.insert(image2) {
                        self.pending.push_back(image2);
                    }
                    let image3 = self.map_handle.beta::<2>(d);
                    if self.marked.insert(image3) {
                        self.pending.push_back(image3);
                    }
                }
                // B1, B2
                OrbitPolicy::VolumeLinear => {
                    let image1 = self.map_handle.beta::<1>(d);
                    if self.marked.insert(image1) {
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<2>(d);
                    if self.marked.insert(image2) {
                        self.pending.push_back(image2);
                    }
                }
                OrbitPolicy::Custom(beta_slice) => {
                    for beta_id in beta_slice {
                        let image = self.map_handle.beta_rt(*beta_id, d);
                        if self.marked.insert(image) {
                            self.pending.push_back(image);
                        }
                    }
                }
            }
            Some(d)
        } else {
            None
        }
    }
}

// --

#[allow(unused_mut)]
#[cfg(test)]
mod tests {}
