//! Orbit implementation
//!
//! This module contains all code used to model orbits, a notion defined
//! along the structure of combinatorial maps.

use std::collections::{HashSet, VecDeque};

use crate::cmap::{CMap2, DartIdType, NULL_DART_ID, OrbitPolicy};
use crate::geometry::CoordsFloat;

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
                    OrbitPolicy::Vertex => {
                        // THIS CODE IS ONLY VALID IN 2D
                        let image1 = self.beta::<1>(self.beta::<2>(d));
                        if marked.insert(image1) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<2>(self.beta::<0>(d));
                        if marked.insert(image2) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image2);
                        }
                    }
                    OrbitPolicy::VertexLinear => {
                        // THIS CODE IS ONLY VALID IN 2D
                        let image = self.beta::<1>(self.beta::<2>(d));
                        if marked.insert(image) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image);
                        }
                    }
                    OrbitPolicy::Edge => {
                        // THIS CODE IS ONLY VALID IN 2D
                        let image = self.beta::<2>(d);
                        if marked.insert(image) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image);
                        }
                    }
                    OrbitPolicy::Face => {
                        // THIS CODE IS ONLY VALID IN 2D
                        let image1 = self.beta::<1>(d);
                        if marked.insert(image1) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image1);
                        }
                        let image2 = self.beta::<0>(d);
                        if marked.insert(image2) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image2);
                        }
                    }
                    OrbitPolicy::FaceLinear => {
                        // THIS CODE IS ONLY VALID IN 2D
                        let image = self.beta::<1>(d);
                        if marked.insert(image) {
                            // if true, we did not see this dart yet
                            // i.e. we need to visit it later
                            pending.push_back(image);
                        }
                    }
                    OrbitPolicy::Custom(beta_slice) => {
                        for beta_id in beta_slice {
                            let image = self.beta_rt(*beta_id, d);
                            if marked.insert(image) {
                                // if true, we did not see this dart yet
                                // i.e. we need to visit it later
                                pending.push_back(image);
                            }
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

    /// Return the orbit defined by a dart and its `I`-cell.
    ///
    /// # Usage
    ///
    /// The [`Orbit2`] can be iterated upon to retrieve all dart member of the cell. Note that
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

#[allow(unused_mut)]
#[cfg(test)]
mod tests {
    use super::*;

    fn simple_map() -> CMap2<f64> {
        let mut map: CMap2<f64> = CMap2::new(11);
        // tri1
        map.force_link::<1>(1, 2).unwrap();
        map.force_link::<1>(2, 3).unwrap();
        map.force_link::<1>(3, 1).unwrap();
        // tri2
        map.force_link::<1>(4, 5).unwrap();
        map.force_link::<1>(5, 6).unwrap();
        map.force_link::<1>(6, 4).unwrap();
        // pent on top
        map.force_link::<1>(7, 8).unwrap();
        map.force_link::<1>(8, 9).unwrap();
        map.force_link::<1>(9, 10).unwrap();
        map.force_link::<1>(10, 11).unwrap();
        map.force_link::<1>(11, 7).unwrap();

        // link all
        map.force_link::<2>(2, 4).unwrap();
        map.force_link::<2>(6, 7).unwrap();

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
        let orbit = map.orbit(OrbitPolicy::Custom(&[1, 2]), 3);
        let darts: Vec<DartIdType> = orbit.collect();
        assert_eq!(darts.len(), 11);
        // because the algorithm is consistent, we can predict the exact layout
        assert_eq!(&darts, &[3, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn orbit_variants() {
        let map = simple_map();

        // face is complete, so everything works
        let face: Vec<DartIdType> = map.orbit(OrbitPolicy::Face, 7).collect();
        let face_linear: Vec<DartIdType> = map.orbit(OrbitPolicy::FaceLinear, 7).collect();
        let face_custom: Vec<DartIdType> = map.orbit(OrbitPolicy::Custom(&[0, 1]), 7).collect();
        assert_eq!(&face, &[7, 8, 11, 9, 10]);
        assert_eq!(&face_linear, &[7, 8, 9, 10, 11]);
        assert_eq!(&face_custom, &[7, 11, 8, 10, 9]);

        // vertex is incomplete, so using the linear variant will yield an incomplete orbit
        let vertex: Vec<DartIdType> = map.orbit(OrbitPolicy::Vertex, 4).collect();
        let vertex_linear: Vec<DartIdType> = map.orbit(OrbitPolicy::VertexLinear, 4).collect();
        assert_eq!(&vertex, &[4, 3, 7]);
        assert_eq!(&vertex_linear, &[4, 3]);
    }

    #[test]
    fn face_from_orbit() {
        let map = simple_map();
        let face_orbit = map.orbit(OrbitPolicy::Face, 1);
        let darts: Vec<DartIdType> = face_orbit.collect();
        assert_eq!(darts.len(), 3);
        assert_eq!(&darts, &[1, 2, 3]);
        let other_face_orbit = map.orbit(OrbitPolicy::Custom(&[1]), 5);
        let other_darts: Vec<DartIdType> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 3);
        assert_eq!(&other_darts, &[5, 6, 4]);
    }

    #[test]
    fn edge_from_orbit() {
        let map = simple_map();
        let face_orbit = map.orbit(OrbitPolicy::Edge, 1);
        let darts: Vec<DartIdType> = face_orbit.collect();
        assert_eq!(darts.len(), 1);
        assert_eq!(&darts, &[1]); // dart 1 is on the boundary
        let other_face_orbit = map.orbit(OrbitPolicy::Custom(&[2]), 4);
        let other_darts: Vec<DartIdType> = other_face_orbit.collect();
        assert_eq!(other_darts.len(), 2);
        assert_eq!(&other_darts, &[4, 2]);
    }

    #[test]
    fn vertex_from_orbit() {
        let map = simple_map();
        let orbit = map.orbit(OrbitPolicy::Vertex, 4);
        let darts: Vec<DartIdType> = orbit.collect();
        assert_eq!(darts.len(), 3);
        assert_eq!(&darts, &[4, 3, 7]);
    }

    #[test]
    fn empty_orbit_policy() {
        let map = simple_map();
        let darts: Vec<DartIdType> = map.orbit(OrbitPolicy::Custom(&[]), 3).collect();
        assert_eq!(&darts, &[3]);
    }

    #[test]
    #[should_panic(expected = "assertion failed: i < 3")]
    fn invalid_orbit_policy() {
        let map = simple_map();
        let orbit = map.orbit(OrbitPolicy::Custom(&[6]), 3);
        let _: Vec<DartIdType> = orbit.collect();
    }
}
