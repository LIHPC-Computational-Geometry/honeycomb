use crate::{CoordsFloat, DartIdentifier, TwoMap};
use std::collections::BTreeSet;

struct OrbitCore<'a, const N_MARKS: usize, T: CoordsFloat> {
    map_handle: &'a TwoMap<N_MARKS, T>,
    beta_slice: &'a [u8],
    visited: BTreeSet<DartIdentifier>,
    pending: Vec<DartIdentifier>,
}

pub struct Orbit<'a, const N_MARKS: usize, T: CoordsFloat> {
    ocore: OrbitCore<'a, N_MARKS, T>,
    current_dart: DartIdentifier,
}

impl<'a, const N_MARKS: usize, T: CoordsFloat> Iterator for Orbit<'a, N_MARKS, T> {
    type Item = DartIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
