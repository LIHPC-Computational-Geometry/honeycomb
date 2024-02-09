//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

use crate::{Dart, DartIdentifier, FaceIdentifier, VertexIdentifier};

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
/// // TODO: two examples to rule them all
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
    betas: Vec<[DartIdentifier; 2]>,
    /// List of free darts identifiers, i.e. empty spots
    /// in the current dart list.
    free_darts: Vec<DartIdentifier>,
}

impl TwoMap {
    /// Creates a new 2D combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `n_darts: usize` -- Number of darts composing the new combinatorial map.
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
        darts.extend((1..n_darts as DartIdentifier + 1).map(Dart::from));

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

    /// Compute the value of the I-th beta function of a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart of interest.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should
    /// be either 1 or 2 in the case of a 2D map.
    ///
    /// # Return / Panic
    ///
    /// Return the dart *d* such that *d = beta_I(dart)*. If the
    /// returned value is the null dart, this means that *dart* is
    /// I-free.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn beta<const I: u8>(&self, dart_id: DartIdentifier) -> DartIdentifier {
        assert!(I < 2);
        assert!(I > 0);
        self.betas[dart_id as usize][(I - 1) as usize]
    }

    /// Fetch cells associated to a given dart.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart of interest.
    ///
    /// # Return / Panic
    ///
    /// Return a [DartCells] structure that contain identifiers to
    /// the different i-cells the dart models.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn cell_of(&self, dart_id: DartIdentifier) -> DartCells {
        self.cells[dart_id as usize]
    }

    /// Check if a given dart is I-free.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart of interest.
    ///
    /// # Return / Panic
    ///
    /// Return a boolean indicating if the dart is I-free.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn is_i_free<const I: u8>(&self, dart_id: DartIdentifier) -> bool {
        self.beta::<I>(dart_id) == 0
    }

    /// Check if a given dart is I-free, for all I.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart of interest.
    ///
    /// # Return / Panic
    ///
    /// Return a boolean indicating if the dart is 1-free and 2-free.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn is_free(&self, dart_id: DartIdentifier) -> bool {
        self.beta::<1>(dart_id) == 0 && self.beta::<2>(dart_id) == 0
    }

    // --- editing interfaces

    /// Add a new free dart to the combinatorial map.
    ///
    /// The dart is I-free for all I and is pushed to the list of existing
    /// darts, effectively making its identifier equal to the total number
    /// of darts.
    ///
    /// # Return / Panic
    ///
    /// Return the created dart to allow for direct operations.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn add_free_dart(&mut self) -> DartIdentifier {
        let new_id = self.darts.len() as DartIdentifier;
        self.darts.push(Dart::from(new_id));
        self.cells.push(DartCells::NULL);
        self.betas.push([0; 2]);
        new_id
    }

    /// Insert a new free dart to the combinatorial map.
    ///
    /// The dart is I-free for all I and may be inserted into a free spot in
    /// the existing dart list. If no free spots exist, it will be pushed to
    /// the end of the list.
    ///
    /// # Return / Panic
    ///
    /// Return the created dart to allow for direct operations.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn insert_free_dart(&mut self) -> DartIdentifier {
        if let Some(new_id) = self.free_darts.pop() {
            self.darts[new_id as usize] = Dart::from(new_id);
            self.cells[new_id as usize] = DartCells::NULL;
            self.betas[new_id as usize] = [0; 2];
            new_id
        } else {
            self.add_free_dart()
        }
    }

    /// Remove a free dart from the combinatorial map.
    ///
    /// The removed dart identifier is added to the list of free dart.
    /// This way of proceeding is necessary as the structure relies on
    /// darts indexing for encoding data, making reordering of any sort
    /// extremely costly.
    ///
    /// By keeping track of free spots in the dart arrays, we can prevent too
    /// much memory waste, although at the cost of locality of reference.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart to remove.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn remove_free_dart(&mut self, dart_id: DartIdentifier) {
        assert!(self.is_free(dart_id));
        self.free_darts.push(dart_id);
        self.betas[dart_id as usize] = [0; 2];
        self.cells[dart_id as usize] = DartCells::NULL;
        self.darts[dart_id as usize] = Dart::NULL;
    }

    /// i-sewing operation.
    ///
    /// This operation corresponds to *coherently linking* two darts via
    /// the beta_I function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart: Dart` -- First dart to be linked.
    /// - `rhs_dart: Dart` -- Second dart to be linked.
    ///
    /// After the sewing operation, these darts will verify
    /// `beta_I(lhs_dart) == rhs_dart`.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should
    /// be either 1 or 2 in the case of a 2D map.
    ///
    /// # Return / Panic
    ///
    /// The method may panic if *I* is neither 1 or 2.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn i_sew<const I: u8>(&mut self, lhs_dart_id: DartIdentifier, rhs_dart_id: DartIdentifier) {
        match I {
            1 => todo!(),
            2 => todo!(),
            _ => panic!(),
        }
    }

    /// i-unsewing operation.
    ///
    /// This operation corresponds to *coherently separating* two darts linked
    /// via the beta_I function. For a thorough explanation of this operation
    /// (and implied hypothesis & consequences), refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
    ///
    /// # Arguments
    ///
    /// - `lhs_dart: Dart` -- Dart to separate.
    ///
    /// Note that we do not need to take two darts as arguments since the
    /// second dart can be obtained through the beta_I function.
    ///
    /// ## Generics
    ///
    /// - `const I: u8` -- Index of the beta function. *I* should
    /// be either 1 or 2 in the case of a 2D map.
    ///
    /// # Return / Panic
    ///
    /// The method may panic if *I* is neither 1 or 2.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn i_unsew<const I: u8>(&mut self, lhs_dart_id: DartIdentifier) {
        match I {
            1 => todo!(),
            2 => todo!(),
            _ => panic!(),
        }
    }

    /// Set the values of the betas function of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart of interest.
    /// - `betas: [usize; 2]` -- Value of the images as `[beta_1(dart), beta_2(dart)]`
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_d_betas(&mut self, dart_id: DartIdentifier, betas: [DartIdentifier; 2]) {
        self.betas[dart_id as usize] = betas;
    }

    /// Set the vertex ID associated to a dart.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart of interest.
    /// - `vertex_id: usize` -- Unique vertex identifier.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_d_vertex(&mut self, dart_id: DartIdentifier, vertex_id: VertexIdentifier) {
        self.cells[dart_id as usize].vertex_id = vertex_id;
    }

    /// Set the face ID associated to a dart.
    ///
    /// # Arguments
    ///
    /// - `dart: Dart` -- Dart of interest.
    /// - `face_id: usize` -- Unique face identifier.
    ///
    /// # Example
    ///
    /// See [TwoMap] example.
    ///
    pub fn set_d_face(&mut self, dart_id: DartIdentifier, face_id: FaceIdentifier) {
        self.cells[dart_id as usize].face_id = face_id;
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
