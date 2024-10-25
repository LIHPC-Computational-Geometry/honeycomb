// ------ IMPORTS

use std::{
    ops::{Index, IndexMut},
    sync::atomic::{AtomicBool, AtomicU32},
};

use stm::TVar;

use crate::cmap::{DartIdentifier, NULL_DART_ID};

// ------ CONTENT

// --- beta functions storage

/// Beta functions storage.
///
/// `N` is the number of beta function stored, including `B0`.
pub struct BetaFunctions<const N: usize>(Vec<[TVar<DartIdentifier>; N]>);

/// Generate beta functions default value for a new dart.
fn new_beta_entry<const N: usize>() -> [TVar<DartIdentifier>; N] {
    (0..N)
        .map(|_| TVar::new(NULL_DART_ID))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

impl<const N: usize> BetaFunctions<N> {
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self((0..=n_darts).map(|_| new_beta_entry()).collect())
    }

    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0.extend((0..len).map(|_| new_beta_entry()));
    }

    /// Return internal storage length
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<const N: usize> Index<(u8, DartIdentifier)> for BetaFunctions<N> {
    type Output = TVar<DartIdentifier>;

    fn index(&self, (beta_id, dart_id): (u8, DartIdentifier)) -> &Self::Output {
        &self.0[dart_id as usize][beta_id as usize]
    }
}

impl<const N: usize> IndexMut<(u8, DartIdentifier)> for BetaFunctions<N> {
    fn index_mut(&mut self, (beta_id, dart_id): (u8, DartIdentifier)) -> &mut Self::Output {
        &mut self.0[dart_id as usize][beta_id as usize]
    }
}

// --- unused darts storage

pub struct UnusedDarts(Vec<AtomicBool>);

impl UnusedDarts {
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self((0..=n_darts).map(|_| AtomicBool::new(false)).collect())
    }

    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0.extend((0..len).map(|_| AtomicBool::new(false)));
    }

    /// Return internal storage length
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<DartIdentifier> for UnusedDarts {
    type Output = AtomicBool;

    fn index(&self, dart_id: DartIdentifier) -> &Self::Output {
        &self.0[dart_id as usize]
    }
}

impl IndexMut<DartIdentifier> for UnusedDarts {
    fn index_mut(&mut self, dart_id: DartIdentifier) -> &mut Self::Output {
        &mut self.0[dart_id as usize]
    }
}
