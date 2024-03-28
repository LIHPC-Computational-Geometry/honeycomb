//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, DartIdentifier, NULL_DART_ID};

use num::Zero;
use std::collections::{BTreeSet, VecDeque};

// ------ CONTENT

/// Enum used to model beta functions defining the search.
///
/// This is used to define special cases of orbits that are often used in
/// algorithms. These special cases correspond to *i-cells*.
pub enum OrbitPolicy<'a> {
    /// 0-cell orbit.
    Vertex,
    /// 1-cell orbit.
    Edge,
    /// 2-cell orbit.
    Face,
    /// Ordered array of beta functions that define the orbit.
    Custom(&'a [u8]),
}

/// Generic 2D orbit implementation
///
/// This structure only contains meta-data about the orbit in its initial
/// state. All the darts making up the orbit are computed when using the
/// methods that come with the [Iterator] implementation.
///
/// It is not currently possible to iterate over references, the orbit has
/// to be consumed for its result to be used. This is most likely the best
/// behavior
///
/// # Generics
///
/// - `'a` -- Lifetime of the reference to the map
/// - `T: CoordsFloat` -- Generic parameter of the referenced map.
///
/// # The search algorithm
///
/// The search algorithm used to establish the list of dart included in the
/// orbit is a [Breadth-first search algorithm][WIKIBFS]. This means that:
///
/// - we look at the images of the current dart through all beta functions,
/// adding those to a queue, before moving on to the next dart.
/// - we apply the beta functions in their specified order (in the case of a
/// custom [OrbitPolicy]); This guarantees a consistent and predictable result.
///
/// Both of these points allow the structure to be used for sewing operations
/// at the cost of some performance (non-trivial parallelization & sequential
/// consistency requirements).
///
/// [WIKIBFS]: https://en.wikipedia.org/wiki/Breadth-first_search
///
/// # Example
///
/// See [CMap2] example.
///
pub struct Orbit2<'a, T: CoordsFloat> {
    /// Reference to the map containing the beta functions used in the BFS.
    map_handle: &'a CMap2<T>,
    /// Policy used by the orbit for the BFS. It can be predetermined or custom.
    orbit_policy: OrbitPolicy<'a>,
    /// Set used to identify which dart is marked during the BFS.
    marked: BTreeSet<DartIdentifier>,
    /// Queue used to store which dart must be visited next during the BFS.
    pending: VecDeque<DartIdentifier>,
}

impl<'a, T: CoordsFloat> Orbit2<'a, T> {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `map_handle: &'a CMap2<T>` -- Reference to the map containing the beta
    /// functions used in the BFS.
    /// - `orbit_policy: OrbitPolicy<'a>` -- Policy used by the orbit for the BFS.
    /// - `dart: DartIdentifier` -- Dart of which the structure will compute the orbit.
    ///
    /// # Return / Panic
    ///
    /// Return an [Orbit2] structure that can be iterated upon to retrieve the orbit's darts.
    ///
    /// The method may panic if no beta index is passed along the custom policy. Additionally,
    /// if an invalid beta index is passed through the custom policy (e.g. `3` for a 2D map),
    /// a panic will occur on iteration
    ///
    /// # Example
    ///
    /// See [CMap2] example.
    ///
    pub fn new(
        map_handle: &'a CMap2<T>,
        orbit_policy: OrbitPolicy<'a>,
        dart: DartIdentifier,
    ) -> Self {
        let mut marked = BTreeSet::<DartIdentifier>::new();
        marked.insert(NULL_DART_ID); // we don't want to include the null dart in the orbit
        marked.insert(dart); // we're starting here, so we mark it beforehand
        let pending = VecDeque::from([dart]);

        if let OrbitPolicy::Custom(slice) = orbit_policy {
            assert!(!slice.len().is_zero());
        }

        Self {
            map_handle,
            orbit_policy,
            marked,
            pending,
        }
    }
}

impl<'a, T: CoordsFloat> Iterator for Orbit2<'a, T> {
    type Item = DartIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(d) = self.pending.pop_front() {
            match self.orbit_policy {
                OrbitPolicy::Vertex => {
                    // THIS CODE IS ONLY VALID IN 2D
                    // WE ASSUME THAT THE EDGE IS COMPLETE
                    let image = self.map_handle.beta::<1>(self.map_handle.beta::<2>(d));
                    if self.marked.insert(image) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image);
                    }
                    Some(d)
                }
                OrbitPolicy::Edge => {
                    // THIS CODE IS ONLY VALID IN 2D
                    let image = self.map_handle.beta::<2>(d);
                    if self.marked.insert(image) {
                        // if true, we did not see this dart yet
                        // i.e. we need to visit it later
                        self.pending.push_back(image);
                    }
                    Some(d)
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
                    Some(d)
                }
                OrbitPolicy::Custom(beta_slice) => {
                    beta_slice.iter().for_each(|beta_id| {
                        let image = self.map_handle.beta_runtime(*beta_id, d);
                        if self.marked.insert(image) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            self.pending.push_back(image);
                        }
                    });

                    Some(d)
                }
            }
        } else {
            None
        }
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use crate::{CMap2, DartIdentifier, FloatType, Orbit2, OrbitPolicy};

    fn simple_map() -> CMap2<FloatType> {
        let mut map: CMap2<FloatType> = CMap2::new(6, 4);
        map.set_betas(1, [3, 2, 0]);
        map.set_betas(2, [1, 3, 4]);
        map.set_betas(3, [2, 1, 0]);
        map.set_betas(4, [6, 5, 2]);
        map.set_betas(5, [4, 6, 0]);
        map.set_betas(6, [5, 4, 0]);
        map.set_vertex(0, (0.0, 0.0)).unwrap();
        map.set_vertex(1, (1.0, 0.0)).unwrap();
        map.set_vertex(2, (1.0, 1.0)).unwrap();
        map.set_vertex(3, (0.0, 1.0)).unwrap();
        map.set_vertexid(1, 0);
        map.set_vertexid(2, 1);
        map.set_vertexid(3, 3);
        map.set_vertexid(4, 3);
        map.set_vertexid(5, 1);
        map.set_vertexid(6, 2);
        map
    }

    #[test]
    fn full_map_from_orbit() {
        let map = simple_map();
        let orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[1, 2]), 3);
        let darts: Vec<DartIdentifier> = orbit.collect();
        assert_eq!(darts.len(), 6);
        // because the algorithm is consistent, we can predict the exact layout
        assert_eq!(&darts, &[3, 1, 2, 4, 5, 6]);
    }

    #[test]
    fn face_from_orbit() {
        let map = simple_map();
        let face_orbit = Orbit2::new(&map, OrbitPolicy::Face, 1);
        let darts: Vec<DartIdentifier> = face_orbit.collect();
        assert_eq!(darts.len(), 3);
        assert_eq!(&darts, &[1, 2, 3]);
        let other_face_orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[1]), 5);
        let other_darts: Vec<DartIdentifier> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 3);
        assert_eq!(&other_darts, &[5, 6, 4]);
    }

    #[test]
    fn edge_from_orbit() {
        let map = simple_map();
        let face_orbit = Orbit2::new(&map, OrbitPolicy::Edge, 1);
        let darts: Vec<DartIdentifier> = face_orbit.collect();
        assert_eq!(darts.len(), 1);
        assert_eq!(&darts, &[1]); // dart 1 is on the boundary
        let other_face_orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[2]), 4);
        let other_darts: Vec<DartIdentifier> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 2);
        assert_eq!(&other_darts, &[4, 2]);
    }

    #[test]
    fn vertex_from_orbit() {
        let map = simple_map();
        let orbit = Orbit2::new(&map, OrbitPolicy::Vertex, 4);
        let darts: Vec<DartIdentifier> = orbit.collect();
        // note that this one fails if we start at 3, because the vertex is not complete
        assert_eq!(darts.len(), 2);
        assert_eq!(&darts, &[4, 3]);
    }

    #[test]
    #[should_panic]
    fn empty_orbit_policy() {
        let map = simple_map();
        let _ = Orbit2::new(&map, OrbitPolicy::Custom(&[]), 3);
    }

    #[test]
    #[should_panic]
    fn invalid_orbit_policy() {
        let map = simple_map();
        let orbit = Orbit2::new(&map, OrbitPolicy::Custom(&[6]), 3);
        let _: Vec<DartIdentifier> = orbit.collect();
    }
}
