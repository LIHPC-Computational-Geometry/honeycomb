use std::{
    ops::{Index, IndexMut},
    slice::Iter,
};

#[cfg(feature = "par-internals")]
use rayon::prelude::*;

use crate::stm::TVar;

use super::identifiers::DartIdType;

/// Unused dart tracking structure.
pub struct UnusedDarts(Vec<TVar<bool>>);

#[allow(unused)]
impl UnusedDarts {
    #[cfg(not(feature = "par-internals"))]
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self((0..n_darts).map(|_| TVar::new(false)).collect())
    }

    #[cfg(feature = "par-internals")]
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self(
            (0..n_darts)
                .into_par_iter()
                .map(|_| TVar::new(false))
                .collect(),
        )
    }

    #[cfg(not(feature = "par-internals"))]
    /// Extend internal storage capacity
    pub fn extend_with(&mut self, len: usize, val: bool) {
        self.0.extend((0..len).map(|_| TVar::new(val)));
    }

    #[cfg(feature = "par-internals")]
    /// Extend internal storage capacity
    pub fn extend_with(&mut self, len: usize, val: bool) {
        self.0
            .par_extend((0..len).into_par_iter().map(|_| TVar::new(val)));
    }

    /// Return internal storage length
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> Iter<'_, TVar<bool>> {
        self.0.iter()
    }

    #[cfg(feature = "par-internals")]
    pub fn par_iter(&self) -> rayon::slice::Iter<'_, TVar<bool>> {
        self.0.par_iter()
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
