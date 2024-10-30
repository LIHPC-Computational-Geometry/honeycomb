// ------ IMPORTS

use std::ops::{Index, IndexMut};

use stm::TVar;

use super::identifiers::{DartId, NULL_DART_ID};

// ------ CONTENT

/// Unused dart tracking structure.
pub struct UnusedDarts(Vec<TVar<bool>>);

impl UnusedDarts {
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self((0..=n_darts).map(|_| TVar::new(false)).collect())
    }

    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0.extend((0..len).map(|_| TVar::new(false)));
    }

    /// Return internal storage length
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<DartId> for UnusedDarts {
    type Output = TVar<bool>;

    fn index(&self, dart_id: DartId) -> &Self::Output {
        &self.0[dart_id.0 as usize]
    }
}

impl IndexMut<DartId> for UnusedDarts {
    fn index_mut(&mut self, dart_id: DartId) -> &mut Self::Output {
        &mut self.0[dart_id.0 as usize]
    }
}
