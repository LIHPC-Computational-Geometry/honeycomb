//! Utility for sample map generation
//!
//! This module contains code used for sample map / mesh generation. This is mostly
//! for testing and benchmarking, but could also be hijacked for very (topologically)
//! simple mesh generation, hence this being kept public.

// ------ IMPORTS

use crate::{CMap2, CMapBuilder, CoordsFloat};

// ------ CONTENT

// --- PUBLIC API

/// Builder structure for specialized [`CMap2`].
///
/// <div class="warning">
///
/// This structure will be reneamed to [`GridDescriptor`] & have its `build2` removed.
///
/// </div>
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
#[deprecated(note = "please use `GridDescriptor` with `CMapBuilder::build` instead")]
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
#[allow(deprecated)]
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
