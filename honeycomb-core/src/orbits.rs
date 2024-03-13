//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CoordsFloat, DartIdentifier, TwoMap};
use std::collections::{BTreeSet, VecDeque};

// ------ CONTENT

struct OrbitCore<'a, const N_MARKS: usize, T: CoordsFloat> {
    pub map_handle: &'a TwoMap<N_MARKS, T>,
    pub beta_slice: &'a [u8],
    pub visited: BTreeSet<DartIdentifier>,
    pub pending: VecDeque<DartIdentifier>,
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

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
