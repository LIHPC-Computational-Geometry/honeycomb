//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CoordsFloat, DartIdentifier, TwoMap, NULL_DART_ID};
use std::collections::{BTreeSet, VecDeque};

// ------ CONTENT

pub enum OrbitPolicy<'a> {
    Vertex,
    Edge,
    Face,
    Custom(&'a [u8]),
}

pub struct Orbit<'a, const N_MARKS: usize, T: CoordsFloat> {
    map_handle: &'a TwoMap<N_MARKS, T>,
    orbit_policy: OrbitPolicy<'a>,
    marked: BTreeSet<DartIdentifier>,
    pending: VecDeque<DartIdentifier>,
}

impl<'a, const N_MARKS: usize, T: CoordsFloat> Orbit<'a, N_MARKS, T> {
    pub fn new(
        map_handle: &'a TwoMap<N_MARKS, T>,
        orbit_policy: OrbitPolicy<'a>,
        dart: DartIdentifier,
    ) -> Self {
        let mut marked = BTreeSet::<DartIdentifier>::new();
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

impl<'a, const N_MARKS: usize, T: CoordsFloat> Iterator for Orbit<'a, N_MARKS, T> {
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
                        let image = self.map_handle.beta_bis(*beta_id, d);
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
    use crate::orbits::OrbitPolicy;
    use crate::{DartIdentifier, FloatType, Orbit, TwoMap};

    fn simple_map() -> TwoMap<1, FloatType> {
        let mut map: TwoMap<1, FloatType> = TwoMap::new(6, 4);
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
        let orbit = Orbit::new(&map, OrbitPolicy::Custom(&[1, 2]), 3);
        let darts: Vec<DartIdentifier> = orbit.into_iter().collect();
        assert_eq!(darts.len(), 6);
        // because the algorithm is consistent, we can predict the exact layout
        assert_eq!(&darts, &[3, 1, 2, 4, 5, 6]);
    }

    #[test]
    fn face_from_orbit() {
        let map = simple_map();
        let face_orbit = Orbit::new(&map, OrbitPolicy::Face, 1);
        let darts: Vec<DartIdentifier> = face_orbit.into_iter().collect();
        assert_eq!(darts.len(), 3);
        assert_eq!(&darts, &[1, 2, 3]);
        let other_face_orbit = Orbit::new(&map, OrbitPolicy::Custom(&[1]), 5);
        let other_darts: Vec<DartIdentifier> = other_face_orbit.into_iter().collect();
        assert_eq!(other_darts.len(), 3);
        assert_eq!(&other_darts, &[5, 6, 4]);
    }

    #[test]
    fn edge_from_orbit() {
        let map = simple_map();
        let face_orbit = Orbit::new(&map, OrbitPolicy::Edge, 1);
        let darts: Vec<DartIdentifier> = face_orbit.into_iter().collect();
        assert_eq!(darts.len(), 1);
        assert_eq!(&darts, &[1]); // dart 1 is on the boundary
        let other_face_orbit = Orbit::new(&map, OrbitPolicy::Custom(&[2]), 4);
        let other_darts: Vec<DartIdentifier> = other_face_orbit.into_iter().collect();
        assert_eq!(other_darts.len(), 2);
        assert_eq!(&other_darts, &[4, 2]);
    }

    #[test]
    fn vertex_from_orbit() {
        let map = simple_map();
        let orbit = Orbit::new(&map, OrbitPolicy::Vertex, 4);
        let darts: Vec<DartIdentifier> = orbit.into_iter().collect();
        // note that this one fails if we start at 3, because the vertex is not complete
        assert_eq!(darts.len(), 2);
        assert_eq!(&darts, &[4, 3]);
    }
}
