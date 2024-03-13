//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CoordsFloat, DartIdentifier, TwoMap};
use std::collections::{BTreeSet, VecDeque};

// ------ CONTENT

pub struct Orbit<'a, const N_MARKS: usize, T: CoordsFloat> {
    map_handle: &'a TwoMap<N_MARKS, T>,
    pub beta_slice: &'a [u8],
    marked: BTreeSet<DartIdentifier>,
    pending: VecDeque<DartIdentifier>,
}

impl<'a, const N_MARKS: usize, T: CoordsFloat> Iterator for Orbit<'a, N_MARKS, T> {
    type Item = DartIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(d) = self.pending.pop_front() {
            self.beta_slice.iter().for_each(|beta_id| {
                let image = self.map_handle.beta_bis(*beta_id, d);
                if self.marked.insert(image) {
                    // if true, we did not see this dart yet
                    // i.e. we need to visit it later
                    self.pending.push_back(image);
                }
            });

            Some(d)
        } else {
            // this makes the structure reusable
            self.marked.clear();
            self.pending.clear();
            None
        }
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
