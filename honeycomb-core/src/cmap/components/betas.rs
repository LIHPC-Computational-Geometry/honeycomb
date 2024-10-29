// ------ IMPORTS

use std::{
    ops::{Index, IndexMut},
    sync::atomic::{AtomicBool, AtomicU32},
};

use stm::TVar;

use super::identifiers::{DartId, NULL_DART_ID};

// ------ CONTENT

/// Beta functions storage.
///
/// `N` is the number of beta function stored, including `B0`.
pub struct BetaFunctions<const N: usize>(Vec<[TVar<DartId>; N]>);

/// Generate beta functions default value for a new dart.
fn new_beta_entry<const N: usize>() -> [TVar<DartId>; N] {
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

impl<const N: usize> Index<(u8, DartId)> for BetaFunctions<N> {
    type Output = TVar<DartId>;

    fn index(&self, (beta_id, dart_id): (u8, DartId)) -> &Self::Output {
        &self.0[dart_id.0 as usize][beta_id as usize]
    }
}

impl<const N: usize> IndexMut<(u8, DartId)> for BetaFunctions<N> {
    fn index_mut(&mut self, (beta_id, dart_id): (u8, DartId)) -> &mut Self::Output {
        &mut self.0[dart_id.0 as usize][beta_id as usize]
    }
}
