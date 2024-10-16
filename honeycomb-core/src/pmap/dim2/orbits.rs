//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

// ------ IMPORTS

use crate::geometry::CoordsFloat;
use crate::prelude::{DartIdentifier, NULL_DART_ID};

use crate::cmap::OrbitPolicy;
use crate::pmap::dim2::structure::PMap2;
use std::collections::{BTreeSet, VecDeque};
// ------ CONTENT

/// Generic 2D orbit implementation
///
/// This structure only contains meta-data about the orbit in its initial state. All the darts
/// making up the orbit are computed when using the methods that come with the [Iterator]
/// implementation.
///
/// It is not currently possible to iterate over references, the orbit has to be consumed for its
/// result to be used. This is most likely the best behavior since orbits should be consumed upon
/// traversal to avoid inconsistencies created by a later mutable operation on the map.
///
/// # Generics
///
/// - `'a` -- Lifetime of the reference to the map
/// - `T: CoordsFloat` -- Generic parameter of the referenced map.
///
/// # The search algorithm
///
/// The search algorithm used to establish the list of dart included in the orbit is a
/// [Breadth-First Search algorithm][WIKIBFS]. This means that:
///
/// - we look at the images of the current dart through all beta functions,
///   adding those to a queue, before moving on to the next dart.
/// - we apply the beta functions in their specified order (in the case of a
///   custom [`OrbitPolicy`]); This guarantees a consistent and predictable result.
///
/// Both of these points allow orbitd to be used for sewing operations at the cost of some
/// performance (non-trivial parallelization & sequential consistency requirements).
///
/// [WIKIBFS]: https://en.wikipedia.org/wiki/Breadth-first_search
///
/// # Example
///
/// See [`CMap2`] example.
///
pub struct POrbit2<'a, T: CoordsFloat> {
    /// Reference to the map containing the beta functions used in the BFS.
    map_handle: &'a PMap2<T>,
    /// Policy used by the orbit for the BFS. It can be predetermined or custom.
    orbit_policy: OrbitPolicy,
    /// Set used to identify which dart is marked during the BFS.
    marked: BTreeSet<DartIdentifier>,
    /// Queue used to store which dart must be visited next during the BFS.
    pending: VecDeque<DartIdentifier>,
}

impl<'a, T: CoordsFloat> POrbit2<'a, T> {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `map_handle: &'a CMap2<T>` -- Reference to the map containing the beta
    ///   functions used in the BFS.
    /// - `orbit_policy: OrbitPolicy<'a>` -- Policy used by the orbit for the BFS.
    /// - `dart: DartIdentifier` -- Dart of which the structure will compute the orbit.
    ///
    /// # Return
    ///
    /// Return an [Orbit2] structure that can be iterated upon to retrieve the orbit's darts.
    ///
    /// # Panics
    ///
    /// The method may panic if no beta index is passed along the custom policy. Additionally,
    /// if an invalid beta index is passed through the custom policy (e.g. `3` for a 2D map),
    /// a panic will occur on iteration
    ///
    /// # Example
    ///
    /// See [`CMap2`] example.
    ///
    #[must_use = "orbits are lazy and do nothing unless consumed"]
    pub fn new(map_handle: &'a PMap2<T>, orbit_policy: OrbitPolicy, dart: DartIdentifier) -> Self {
        let mut marked = BTreeSet::<DartIdentifier>::new();
        marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit
        marked.insert(dart); // we're starting here, so we mark it beforehand
        let pending = VecDeque::from([dart]);
        /*
        if let OrbitPolicy::Custom(slice) = orbit_policy {
            assert!(!slice.len().is_zero());
        }
        */
        Self {
            map_handle,
            orbit_policy,
            marked,
            pending,
        }
    }
}

impl<'a, T: CoordsFloat> Iterator for POrbit2<'a, T> {
    type Item = DartIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(d) = self.pending.pop_front() {
            match self.orbit_policy {
                OrbitPolicy::Vertex => {
                    // THIS CODE IS ONLY VALID IN 2D

                    let image1 = self.map_handle.beta::<1>(self.map_handle.beta::<2>(d));
                    if self.marked.insert(image1) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<2>(self.map_handle.beta::<0>(d));
                    if self.marked.insert(image2) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image2);
                    }
                }
                OrbitPolicy::Edge => {
                    // THIS CODE IS ONLY VALID IN 2D
                    let image = self.map_handle.beta::<2>(d);
                    if self.marked.insert(image) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image);
                    }
                }
                OrbitPolicy::Face => {
                    // THIS CODE IS ONLY VALID IN 2D
                    // WE ASSUME THAT THE FACE IS COMPLETE
                    let image = self.map_handle.beta::<1>(d);
                    if self.marked.insert(image) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image);
                    }
                }
                OrbitPolicy::Custom(beta_slice) => {
                    for beta_id in beta_slice {
                        let image = self.map_handle.beta_runtime(*beta_id, d);
                        if self.marked.insert(image) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
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

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_map() -> PMap2<f64> {
        let mut map: PMap2<f64> = PMap2::new(6);
        map.one_link(1, 2);
        map.one_link(2, 3);
        map.one_link(3, 1);
        map.one_link(4, 5);
        map.one_link(5, 6);
        map.one_link(6, 4);
        map.two_link(2, 4);
        /*
        assert!(map.replace_vertex(1, (0.0, 0.0)).is_none());
        assert!(map.replace_vertex(2, (1.0, 0.0)).is_none());
        assert!(map.replace_vertex(6, (1.0, 1.0)).is_none());
        assert!(map.replace_vertex(3, (0.0, 1.0)).is_none());

         */
        map
    }

    #[test]
    fn full_map_from_orbit() {
        let map = simple_map();
        let orbit = POrbit2::new(&map, OrbitPolicy::Custom(&[1, 2]), 3);
        let darts: Vec<DartIdentifier> = orbit.collect();
        assert_eq!(darts.len(), 6);
        // because the algorithm is consistent, we can predict the exact layout
        assert_eq!(&darts, &[3, 1, 2, 4, 5, 6]);
    }

    #[test]
    fn face_from_orbit() {
        let map = simple_map();
        let face_orbit = POrbit2::new(&map, OrbitPolicy::Face, 1);
        let darts: Vec<DartIdentifier> = face_orbit.collect();
        assert_eq!(darts.len(), 3);
        assert_eq!(&darts, &[1, 2, 3]);
        let other_face_orbit = POrbit2::new(&map, OrbitPolicy::Custom(&[1]), 5);
        let other_darts: Vec<DartIdentifier> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 3);
        assert_eq!(&other_darts, &[5, 6, 4]);
    }

    #[test]
    fn edge_from_orbit() {
        let map = simple_map();
        let face_orbit = POrbit2::new(&map, OrbitPolicy::Edge, 1);
        let darts: Vec<DartIdentifier> = face_orbit.collect();
        assert_eq!(darts.len(), 1);
        assert_eq!(&darts, &[1]); // dart 1 is on the boundary
        let other_face_orbit = POrbit2::new(&map, OrbitPolicy::Custom(&[2]), 4);
        let other_darts: Vec<DartIdentifier> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 2);
        assert_eq!(&other_darts, &[4, 2]);
    }

    #[test]
    fn vertex_from_orbit() {
        let map = simple_map();
        let orbit = POrbit2::new(&map, OrbitPolicy::Vertex, 4);
        let darts: Vec<DartIdentifier> = orbit.collect();
        // note that this one fails if we start at 3, because the vertex is not complete
        assert_eq!(darts.len(), 2);
        assert_eq!(&darts, &[4, 3]);
    }

    #[test]
    fn empty_orbit_policy() {
        let map = simple_map();
        let darts: Vec<DartIdentifier> = POrbit2::new(&map, OrbitPolicy::Custom(&[]), 3).collect();
        assert_eq!(&darts, &[3]);
    }

    #[test]
    #[should_panic(expected = "assertion failed: i < 3")]
    fn invalid_orbit_policy() {
        let map = simple_map();
        let orbit = POrbit2::new(&map, OrbitPolicy::Custom(&[6]), 3);
        let _: Vec<DartIdentifier> = orbit.collect();
    }
}
