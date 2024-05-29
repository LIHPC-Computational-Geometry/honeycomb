//! Utility for sample map generation
//!
//! This module contains code used to generate maps that represent grids. These have a variety
//! of usages, most notably in tests, benchmarks, and specific algorithms.

// ------ MODULE DECLARATIONS

mod building_routines;
mod descriptor;

// ------ RE-EXPORTS

use crate::{CMapBuilder, CoordsFloat};
pub(super) use building_routines::{build2_grid, build2_splitgrid};
pub use descriptor::GridDescriptor;

// ------ CONTENT

// --- impl items for CMapBuilder

impl<T: CoordsFloat> CMapBuilder<T> {
    #[cfg(feature = "utils")]
    /// Set the [`GridDescriptor`] that will be used when building the map.
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn grid_descriptor(mut self, grid_descriptor: GridDescriptor<T>) -> Self {
        self.grid_descriptor = Some(grid_descriptor);
        self
    }

    #[cfg(feature = "utils")]
    /// Create a [`CMapBuilder`] from a [`GridDescriptor`].
    ///
    /// This function is roughly equivalent to the following:
    ///
    /// ```rust
    /// # use honeycomb_core::{CMapBuilder, GridDescriptor};
    /// // setup grid descriptor
    /// let gridd = GridDescriptor::default();
    /// // ...
    ///
    /// // `CMapBuilder::from_grid_descriptor(gridd)`, or:
    /// let builder = CMapBuilder::<f64>::default().grid_descriptor(gridd);
    /// ```
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn from_grid_descriptor(grid_descriptor: GridDescriptor<T>) -> Self {
        CMapBuilder {
            grid_descriptor: Some(grid_descriptor),
            ..Default::default()
        }
    }
}

// --- predefinite grid setups for CMapBuilder

impl<T: CoordsFloat> CMapBuilder<T> {
    /// Create a [`CMapBuilder`] with a predefinite [`GridDescriptor`] value.
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn unit_grid(n_square: usize) -> Self {
        let gridd = GridDescriptor::default()
            .n_cells([n_square; 3])
            .len_per_cell([T::one(); 3]);
        CMapBuilder::from_grid_descriptor(gridd)
    }

    /// Create a [`CMapBuilder`] with a predefinite [`GridDescriptor`] value.
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn unit_split_grid(n_square: usize) -> Self {
        let gridd = GridDescriptor::default()
            .n_cells([n_square; 3])
            .len_per_cell([T::one(); 3])
            .split_quads(true);
        CMapBuilder::from_grid_descriptor(gridd)
    }
}

// ------ TESTS
#[cfg(test)]
mod tests;
