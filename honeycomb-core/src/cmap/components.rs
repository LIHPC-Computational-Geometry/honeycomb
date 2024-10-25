// ------ IMPORTS

use std::{
    ops::{Index, IndexMut},
    sync::atomic::{AtomicBool, AtomicU32},
};

use crate::cmap::{DartIdentifier, NULL_DART_ID};

// ------ CONTENT

// --- beta functions storage

pub struct BetaFunctions<const N: usize>(Vec<[AtomicU32; N]>);

impl<const N: usize> BetaFunctions<N> {
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self(
            (0..=n_darts)
                .map(|_| [const { AtomicU32::new(NULL_DART_ID) }; N])
                .collect(),
        )
    }

    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0
            .extend((0..len).map(|_| [const { AtomicU32::new(NULL_DART_ID) }; N]));
    }

    /// Return internal storage length
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<const N: usize> Index<(u8, DartIdentifier)> for BetaFunctions<N> {
    type Output = AtomicU32;

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
