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

    // --- editing interfaces

    pub fn add_free_dart(&mut self) -> Dart {
        let new_id = self.darts.len();
        self.darts.push(Dart::from(new_id));
        self.cells.push(DartCells::NULL);
        self.betas.push([0; 2]);
        Dart::from(new_id)
    }

    pub fn insert_free_dart(&mut self) -> Dart {
        if let Some(new_id) = self.free_darts.pop() {
            self.darts[new_id] = Dart::from(new_id);
            self.cells[new_id] = DartCells::NULL;
            self.betas[new_id] = [0; 2];
            Dart::from(new_id)
        } else {
            self.add_free_dart()
        }
    }

    pub fn remove_free_dart(&mut self, dart: Dart) {
        assert!(self.is_free(dart));
        self.free_darts.push(dart.id());
        self.betas[dart.id()] = [0; 2];
        self.cells[dart.id()] = DartCells::NULL;
        self.darts[dart.id()] = Dart::NULL;
    }

    pub fn i_sew<const I: usize>(&mut self, lhs_dart: Dart, rhs_dart: Dart) {
        match I {
            1 => todo!(),
            2 => todo!(),
            _ => panic!(),
        }
    }

    pub fn i_unsew<const I: usize>(&mut self, lhs_dart: Dart) {
        match I {
            1 => todo!(),
            2 => todo!(),
            _ => panic!(),
        }
    }

    pub fn set_d_betas(&mut self, dart: Dart, betas: [usize; 2]) {
        self.betas[dart.id()] = betas;
    }

    pub fn set_d_vertex(&mut self, dart: Dart, vertex_id: usize) {
        self.cells[dart.id()].vertex_id = vertex_id;
    }

    pub fn set_d_face(&mut self, dart: Dart, face_id: usize) {
        self.cells[dart.id()].face_id = face_id;
    }
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
