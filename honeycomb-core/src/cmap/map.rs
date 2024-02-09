//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

use crate::Dart;

use super::embed::DartCells;

// ------ CONTENT

// --- 2-MAP

/// Main map object.
///
/// Detailed description.
///
/// # Example
///
/// ```
/// use honeycomb_core::TwoMap;
///
/// // TODO: one example to rule them all
///
/// ```
///
pub struct TwoMap {
    /// List of associated cells of each dart.
    cells: Vec<DartCells>,
    /// List of darts composing the map.
    ///
    /// Used to ...
    darts: Vec<Dart>,
    /// Array representation of the beta functions.
    ///
    /// This should eventually be replaced by a better
    /// structure, supported by benchmarking.
    betas: Vec<[usize; 2]>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list.
    free_darts: Vec<usize>,
}

impl TwoMap {
    /// Creates a new 2D combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts  composing the new combinatorial map.
    ///
    /// # Return / Panic
    ///
    /// Returns a combinatorial map containing:
    /// - `n_darts + 1`, the amount of darts wanted plus the null dart (at index 0).
    /// - 2 beta functions, stored with an offset of 1 due to the absence of beta 0.
    /// - Default embed data associated to each dart.
    /// - An empty list of currently free darts. This may be used for dart creation.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn new(n_darts: usize) -> Self {
        let mut darts = vec![Dart::NULL];
        darts.extend((1..n_darts + 1).map(Dart::from));

        let cells = vec![DartCells::NULL; n_darts + 1];

        let betas = vec![[0; 2]; n_darts + 1];

        Self {
            cells,
            darts,
            betas,
            free_darts: Vec::with_capacity(n_darts + 1),
        }
    }

    // --- reading interfaces

    pub fn beta<const I: u8>(&self, dart: Dart) -> Dart {
        assert!(I < 2);
        assert!(I > 0);
        Dart::from(self.betas[dart.id()][(I - 1) as usize])
    }

    pub fn cell_of(&self, dart: Dart) -> DartCells {
        self.cells[dart.id()]
    }

    pub fn is_i_free<const I: u8>(&self, dart: Dart) -> bool {
        self.beta::<I>(dart) == Dart::NULL
    }

    pub fn is_free(&self, dart: Dart) -> bool {
        self.beta::<1>(dart) == Dart::NULL && self.beta::<2>(dart) == Dart::NULL
    }

    // -- editing interfaces
}

// --- 3-MAP

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
