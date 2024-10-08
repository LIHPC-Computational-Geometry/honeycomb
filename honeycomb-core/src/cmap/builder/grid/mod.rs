//! Utility for sample map generation
//!
//! This module contains code used to generate maps that represent grids. These have a variety
//! of usages, most notably in tests, benchmarks, and specific algorithms.

// ------ MODULE DECLARATIONS

pub mod building_routines;
pub mod descriptor;

// ------ RE-EXPORTS

use crate::geometry::CoordsFloat;
use crate::prelude::{CMapBuilder, GridDescriptor};

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
}
/// Create a [`CMapBuilder`] from a [`GridDescriptor`].
///
/// This implementation is roughly equivalent to the following:
///
/// ```rust
/// # use honeycomb_core::prelude::{CMapBuilder, GridDescriptor};
/// // setup grid descriptor
/// let gridd = GridDescriptor::default();
/// // ...
///
/// // `CMapBuilder::from(gridd)`, or:
/// let builder = CMapBuilder::<f64>::default().grid_descriptor(gridd);
/// ```
#[cfg(feature = "utils")]
impl<T: CoordsFloat> From<GridDescriptor<T>> for CMapBuilder<T> {
    fn from(value: GridDescriptor<T>) -> Self {
        CMapBuilder {
            grid_descriptor: Some(value),
            ..Default::default()
        }
    }
}

// --- predefinite grid setups for CMapBuilder

impl<T: CoordsFloat> CMapBuilder<T> {
    /// Create a [`CMapBuilder`] with a predefinite [`GridDescriptor`] value.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Number of cells along each axis.
    ///
    /// # Return
    ///
    /// This function return a builder structure with predefinite parameters to generate
    /// a specific map.
    ///
    /// The map generated by this predefinite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis.
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/images/bg_grid.svg)
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn unit_grid(n_square: usize) -> Self {
        GridDescriptor::default()
            .n_cells([n_square; 3])
            .len_per_cell([T::one(); 3])
            .into()
    }

    /// Create a [`CMapBuilder`] with a predefinite [`GridDescriptor`] value.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Number of cells along each axis.
    ///
    /// # Return
    ///
    /// This function return a builder structure with predefinite parameters to generate
    /// a specific map.
    ///
    /// The map generated by this predefinite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis. Each cell will be split across their diagonal (top
    /// left to bottom right) to form triangles.
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/images/bg_grid_tri.svg)
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn unit_triangles(n_square: usize) -> Self {
        GridDescriptor::default()
            .n_cells([n_square; 3])
            .len_per_cell([T::one(); 3])
            .split_quads(true)
            .into()
    }
}

// ------ TESTS
#[cfg(test)]
mod tests;
