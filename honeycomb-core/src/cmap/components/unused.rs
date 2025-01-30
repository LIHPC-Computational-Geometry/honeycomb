// ------ IMPORTS

use std::{
    ops::{Index, IndexMut},
    slice::Iter,
};

use crate::stm::TVar;

use super::identifiers::DartIdType;

// ------ CONTENT

/// Unused dart tracking structure.
pub struct UnusedDarts(Vec<TVar<bool>>);

#[allow(unused)]
impl UnusedDarts {
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self((0..n_darts).map(|_| TVar::new(false)).collect())
    }

    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0.extend((0..len).map(|_| TVar::new(false)));
    }

    /// Return internal storage length
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> Iter<'_, TVar<bool>> {
        self.0.iter()
    }
}

impl Index<DartIdType> for UnusedDarts {
    type Output = TVar<bool>;

    fn index(&self, dart_id: DartIdType) -> &Self::Output {
        &self.0[dart_id as usize]
    }
}

impl IndexMut<DartIdType> for UnusedDarts {
    fn index_mut(&mut self, dart_id: DartIdType) -> &mut Self::Output {
        &mut self.0[dart_id as usize]
    }
}
