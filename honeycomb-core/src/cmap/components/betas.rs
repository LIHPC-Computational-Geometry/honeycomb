// ------ IMPORTS

use std::ops::{Index, IndexMut};

use stm::TVar;

use crate::cmap::NULL_DART_ID;

use super::identifiers::DartIdType;

// ------ CONTENT

/// Beta functions storage.
///
/// `N` is the number of beta function stored, including `B0`. This means that, for example,
/// a 2-map will have a `BetaFunctions<3>` object field.
pub struct BetaFunctions<const N: usize>(Vec<[TVar<DartIdType>; N]>);

/// Generate beta functions default value for a new dart.
fn new_beta_entry<const N: usize>() -> [TVar<DartIdType>; N] {
    (0..N)
        .map(|_| TVar::new(NULL_DART_ID))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

impl<const N: usize> BetaFunctions<N> {
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self((0..n_darts).map(|_| new_beta_entry()).collect())
    }

    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0.extend((0..len).map(|_| new_beta_entry()));
    }

    /// Return internal storage capacity
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<const N: usize> Index<(u8, DartIdType)> for BetaFunctions<N> {
    type Output = TVar<DartIdType>;

    fn index(&self, (beta_id, dart_id): (u8, DartIdType)) -> &Self::Output {
        &self.0[dart_id as usize][beta_id as usize]
    }
}

impl<const N: usize> IndexMut<(u8, DartIdType)> for BetaFunctions<N> {
    fn index_mut(&mut self, (beta_id, dart_id): (u8, DartIdType)) -> &mut Self::Output {
        &mut self.0[dart_id as usize][beta_id as usize]
    }
}
