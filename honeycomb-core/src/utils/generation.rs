//! Utility for sample map generation
//!
//! This module contains code used for sample map / mesh generation. This is mostly
//! for testing and benchmarking, but could also be hijacked for very (topologically)
//! simple mesh generation, hence this being kept public.

// ------ IMPORTS

use crate::{BuilderError, CMap2, CMapBuilder, CoordsFloat};

// ------ CONTENT

// --- PUBLIC API

/// Builder structure for specialized [`CMap2`].
///
/// The user must specify two out of these three characteristics:
///
/// - `n_cells: [usize; 3]` - The number of cells per axis
/// - `len_per_cell: [T; 3]` - The dimensions of cells per axis
/// - `lens: [T; 3]` -The dimensions of the grid per axis
///
/// This can be done using the provided eponymous methods. The structure can then be used to
/// generate a [`CMap2`] using [`GridBuilder::build2`].
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type of the future [`CMap2`] instance.
///
/// # Example
///
/// The following code generates a map that can be visualized by running the example
/// `render_squaremap_parameterized`:
///
/// ```rust
/// # fn main() {
/// use honeycomb_core::{CMap2, utils::GridBuilder};
///
/// let map = GridBuilder::default()
///     .n_cells([15, 5, 0])
///     .len_per_cell_x(1.0_f64)
///     .len_per_cell_y(3.0_f64)
///     .build2();
/// # }
/// ```
///
#[derive(Default, Clone)]
pub struct GridBuilder<T: CoordsFloat> {
    pub(crate) n_cells: Option<[usize; 3]>,
    pub(crate) len_per_cell: Option<[T; 3]>,
    pub(crate) lens: Option<[T; 3]>,
    pub(crate) split_quads: bool,
}

macro_rules! setters {
    ($fld: ident, $fldx: ident, $fldy: ident, $fldz: ident, $zero: expr, $fldty: ty) => {
        /// Set values for all dimensions
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fld(mut self, $fld: [$fldty; 3]) -> Self {
            self.$fld = Some($fld);
            self
        }

        /// Set x-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldx(mut self, $fld: $fldty) -> Self {
            if let Some([ptr, _, _]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$fld, $zero, $zero]);
            }
            self
        }

        /// Set y-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldy(mut self, $fld: $fldty) -> Self {
            if let Some([_, ptr, _]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$zero, $fld, $zero]);
            }
            self
        }

        /// Set z-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldz(mut self, $fld: $fldty) -> Self {
            if let Some([_, _, ptr]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$zero, $zero, $fld]);
            }
            self
        }
    };
}

// editing methods
impl<T: CoordsFloat> GridBuilder<T> {
    // n_cells
    setters!(n_cells, n_cells_x, n_cells_y, n_cells_z, 0, usize);

    // len_per_cell
    setters!(
        len_per_cell,
        len_per_cell_x,
        len_per_cell_y,
        len_per_cell_z,
        T::zero(),
        T
    );

    // lens
    setters!(lens, lens_x, lens_y, lens_z, T::zero(), T);

    /// Indicate whether to split quads of the grid
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn split_quads(mut self, split: bool) -> Self {
        self.split_quads = split;
        self
    }
}

// building methods
impl<T: CoordsFloat> GridBuilder<T> {
    #[allow(clippy::missing_errors_doc)]
    /// Consumes the builder and produce a [`CMap2`] object.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(map: CMap2)` -- The method used two of the three parameters to generate a [`CMap2`]
    /// instance
    /// - `Err(BuilderError::MissingParameters)` -- The provided information was not sufficient to
    /// create an instance
    /// - `Err(BuilderError::InvalidParameters)` -- Any of the used length is negative or null
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    ///
    /// # Example
    ///
    /// See [`GridBuilder`] example.
    ///
    pub fn build2(self) -> Result<CMap2<T>, BuilderError> {
        CMapBuilder::from_grid_descriptor(self).build2()
    }
}

// predefinite constructs
impl<T: CoordsFloat> GridBuilder<T> {
    /// Generate a predefined [`GridBuilder`] object.
    ///
    /// This object can be used to build a 2-map representing an orthogonal mesh made of
    /// `n_square * n_square` square cells.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Dimension of the desired mesh.
    ///
    /// ## Generics
    ///
    /// - `const T: CoordsFloat` -- Generic parameter of the returned [`GridBuilder`].
    ///
    /// # Return
    ///
    /// Returns a parameterized [`GridBuilder`] that can be used to generate a [`CMap2`] using the
    /// [`GridBuilder::build2`] method.
    ///
    /// # Example
    ///
    /// ```
    /// use honeycomb_core::{CMap2, utils::GridBuilder};
    ///
    /// let cmap: CMap2<f64> = GridBuilder::unit_squares(2).build2().unwrap();
    /// ```
    ///
    /// The above code generates the following map:
    ///
    /// ![SQUARECMAP2](../../images/CMap2Square.svg)
    ///
    /// Note that *β<sub>1</sub>* is only represented in one cell but is defined
    /// Everywhere following the same pattern. Dart indexing is also consistent
    /// with the following rules:
    ///
    /// - inside a cell, the first dart is the one on the bottom, pointing towards
    ///   the right. Increments (and *β<sub>1</sub>*) follow the trigonometric
    ///   orientation.
    /// - cells are ordered from left to right, from the bottom up. The same rule
    ///   applies for face IDs.
    ///
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn unit_squares(n_square: usize) -> Self {
        Self {
            n_cells: Some([n_square; 3]),
            len_per_cell: Some([T::one(); 3]),
            ..Default::default()
        }
    }

    /// Generate a predefined [`GridBuilder`] object.
    ///
    /// This object can be used to build a 2-map representing an orthogonal mesh made of
    /// `n_square * n_square` squares, that are split diagonally for a total of
    /// `n_square * n_square * 2` cells.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Dimension of the desired mesh.
    ///
    /// ## Generics
    ///
    /// - `const T: CoordsFloat` -- Generic parameter of the returned [`GridBuilder`].
    ///
    /// # Return
    ///
    /// Returns a parameterized [`GridBuilder`] that can be used to generate a [`CMap2`] using the
    /// [`GridBuilder::build2`] method.
    ///
    /// # Example
    ///
    /// ```
    /// use honeycomb_core::{CMap2, utils::GridBuilder};
    ///
    /// let cmap: CMap2<f64> = GridBuilder::split_unit_squares(2).build2().unwrap();
    /// ```
    ///
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn split_unit_squares(n_square: usize) -> Self {
        Self {
            n_cells: Some([n_square; 3]),
            len_per_cell: Some([T::one(); 3]),
            split_quads: true,
            ..Default::default()
        }
    }
}
