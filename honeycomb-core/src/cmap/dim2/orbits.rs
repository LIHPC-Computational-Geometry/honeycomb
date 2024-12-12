//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

// ------ IMPORTS

use crate::geometry::CoordsFloat;
use crate::prelude::{CMap2, DartIdType, OrbitPolicy, NULL_DART_ID};

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
pub struct Orbit2<'a, T: CoordsFloat> {
    /// Reference to the map containing the beta functions used in the BFS.
    map_handle: &'a CMap2<T>,
    /// Policy used by the orbit for the BFS. It can be predetermined or custom.
    orbit_policy: OrbitPolicy,
    /// Set used to identify which dart is marked during the BFS.
    marked: BTreeSet<DartIdType>,
    /// Queue used to store which dart must be visited next during the BFS.
    pending: VecDeque<DartIdType>,
}

impl<'a, T: CoordsFloat> Orbit2<'a, T> {
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
    pub fn new(map_handle: &'a CMap2<T>, orbit_policy: OrbitPolicy, dart: DartIdType) -> Self {
        let mut marked = BTreeSet::<DartIdType>::new();
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

impl<T: CoordsFloat> Iterator for Orbit2<'_, T> {
    type Item = DartIdType;

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
                OrbitPolicy::VertexLinear => {
                    // THIS CODE IS ONLY VALID IN 2D
                    let image = self.map_handle.beta::<1>(self.map_handle.beta::<2>(d));
                    if self.marked.insert(image) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image);
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
                    let image1 = self.map_handle.beta::<1>(d);
                    if self.marked.insert(image1) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image1);
                    }
                    let image2 = self.map_handle.beta::<0>(d);
                    if self.marked.insert(image2) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image2);
                    }
                }
                OrbitPolicy::FaceLinear => {
                    // THIS CODE IS ONLY VALID IN 2D
                    let image = self.map_handle.beta::<1>(d);
                    if self.marked.insert(image) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image);
                    }
                }
                OrbitPolicy::Custom(beta_slice) => {
                    for beta_id in beta_slice {
                        let image = self.map_handle.beta_rt(*beta_id, d);
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

#[allow(unused_mut)]
#[cfg(test)]
mod tests {
    use super::*;

    fn simple_map() -> CMap2<f64> {
        let mut map: CMap2<f64> = CMap2::new(11);
        // tri1
        map.force_link::<1>(1, 2);
        map.force_link::<1>(2, 3);
        map.force_link::<1>(3, 1);
        // tri2
        map.force_link::<1>(4, 5);
        map.force_link::<1>(5, 6);
        map.force_link::<1>(6, 4);
        // pent on top
        map.force_link::<1>(7, 8);
        map.force_link::<1>(8, 9);
        map.force_link::<1>(9, 10);
        map.force_link::<1>(10, 11);
        map.force_link::<1>(11, 7);

        // link all
        map.force_link::<2>(2, 4);
        map.force_link::<2>(6, 7);

        assert!(map.force_write_vertex(1, (0.0, 0.0)).is_none());
        assert!(map.force_write_vertex(2, (1.0, 0.0)).is_none());
        assert!(map.force_write_vertex(6, (1.0, 1.0)).is_none());
        assert!(map.force_write_vertex(3, (0.0, 1.0)).is_none());
        assert!(map.force_write_vertex(9, (1.5, 1.5)).is_none());
        assert!(map.force_write_vertex(10, (0.5, 2.0)).is_none());
        assert!(map.force_write_vertex(11, (-0.5, 1.5)).is_none());

        map
    }

    #[test]
    fn full_map_from_orbit() {
        let map = simple_map();
        let orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[1, 2]), 3);
        let darts: Vec<DartIdType> = orbit.collect();
        assert_eq!(darts.len(), 11);
        // because the algorithm is consistent, we can predict the exact layout
        assert_eq!(&darts, &[3, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn orbit_variants() {
        let map = simple_map();

        // face is complete, so everything works
        let face: Vec<DartIdType> = Orbit2::new(&map, OrbitPolicy::Face, 7).collect();
        let face_linear: Vec<DartIdType> = Orbit2::new(&map, OrbitPolicy::FaceLinear, 7).collect();
        let face_custom: Vec<DartIdType> =
            Orbit2::new(&map, OrbitPolicy::Custom(&[0, 1]), 7).collect();
        assert_eq!(&face, &[7, 8, 11, 9, 10]);
        assert_eq!(&face_linear, &[7, 8, 9, 10, 11]);
        assert_eq!(&face_custom, &[7, 11, 8, 10, 9]);

        // vertex is incomplete, so using the linear variant will yield an incomplete orbit
        let vertex: Vec<DartIdType> = Orbit2::new(&map, OrbitPolicy::Vertex, 4).collect();
        let vertex_linear: Vec<DartIdType> =
            Orbit2::new(&map, OrbitPolicy::VertexLinear, 4).collect();
        assert_eq!(&vertex, &[4, 3, 7]);
        assert_eq!(&vertex_linear, &[4, 3]);
    }

    #[test]
    fn face_from_orbit() {
        let map = simple_map();
        let face_orbit = Orbit2::new(&map, OrbitPolicy::Face, 1);
        let darts: Vec<DartIdType> = face_orbit.collect();
        assert_eq!(darts.len(), 3);
        assert_eq!(&darts, &[1, 2, 3]);
        let other_face_orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[1]), 5);
        let other_darts: Vec<DartIdType> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 3);
        assert_eq!(&other_darts, &[5, 6, 4]);
    }

    #[test]
    fn edge_from_orbit() {
        let map = simple_map();
        let face_orbit = Orbit2::new(&map, OrbitPolicy::Edge, 1);
        let darts: Vec<DartIdType> = face_orbit.collect();
        assert_eq!(darts.len(), 1);
        assert_eq!(&darts, &[1]); // dart 1 is on the boundary
        let other_face_orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[2]), 4);
        let other_darts: Vec<DartIdType> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 2);
        assert_eq!(&other_darts, &[4, 2]);
    }

    #[test]
    fn vertex_from_orbit() {
        let map = simple_map();
        let orbit = Orbit2::new(&map, OrbitPolicy::Vertex, 4);
        let darts: Vec<DartIdType> = orbit.collect();
        assert_eq!(darts.len(), 3);
        assert_eq!(&darts, &[4, 3, 7]);
    }

    #[test]
    fn empty_orbit_policy() {
        let map = simple_map();
        let darts: Vec<DartIdType> = Orbit2::new(&map, OrbitPolicy::Custom(&[]), 3).collect();
        assert_eq!(&darts, &[3]);
    }

    #[test]
    #[should_panic(expected = "assertion failed: i < 3")]
    fn invalid_orbit_policy() {
        let map = simple_map();
        let orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[6]), 3);
        let _: Vec<DartIdType> = orbit.collect();
    }
}
