//! grid generation implementation
//!
//! This module contains the routines and interface used to implement 2D and 3D grid generation.

mod internals;

#[cfg(test)]
mod tests;

use honeycomb_core::attributes::AttributeBind;
use honeycomb_core::cmap::{CMap2, CMap3, CMapBuilder};
use honeycomb_core::geometry::{CoordsFloat, Vertex2, Vertex3};

/// # Builder-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when using the [`GridBuilder`]
/// structure.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum GridBuilderError {
    // grid-related variants
    /// One or multiple of the specified grid characteristics are invalid.
    #[error("invalid grid parameters - {0}")]
    InvalidGridParameters(&'static str),
    /// The builder is missing one or multiple parameters to generate the grid.
    #[error("insufficient parameters - please specify at least 2")]
    MissingGridParameters,
}

/// # Grid builder structure
///
/// ## Generics
///
/// - `const D: usize` -- Dimension of the grid. Should be 2 or 3.
/// - `T: CoordsFloat` -- Generic FP type that will be used by the map's vertices.
///
/// ## Example
///
/// ```rust
/// # use honeycomb_kernels::grid_generation::{GridBuilderError};
/// # fn main() -> Result<(), GridBuilderError> {
/// use honeycomb_core::cmap::CMap3;
/// use honeycomb_kernels::grid_generation::{GridBuilder};
///
/// let map: CMap3<f64> = GridBuilder::<3, f64>::default()
///     .n_cells([2, 3, 2])
///     .len_per_cell([1.0; 3])
///     .build()?;
///
/// # Ok(())
/// # }
/// ```
pub struct GridBuilder<const D: usize, T: CoordsFloat> {
    pub(crate) map_builder: CMapBuilder<D>,
    pub(crate) origin: [T; D],
    pub(crate) n_cells: Option<[usize; D]>,
    pub(crate) len_per_cell: Option<[T; D]>,
    pub(crate) lens: Option<[T; D]>,
    pub(crate) split_cells: bool,
}

impl<const D: usize, T: CoordsFloat> Default for GridBuilder<D, T> {
    fn default() -> Self {
        Self {
            map_builder: CMapBuilder::<D>::from_n_darts(1),
            origin: [T::zero(); D],
            n_cells: None,
            len_per_cell: None,
            lens: None,
            split_cells: false,
        }
    }
}

impl<const D: usize, T: CoordsFloat> GridBuilder<D, T> {
    /// Set values for all dimensions
    #[must_use = "unused builder object"]
    pub fn n_cells(mut self, n_cells: [usize; D]) -> Self {
        self.n_cells = Some(n_cells);
        self
    }

    /// Set values for all dimensions
    #[must_use = "unused builder object"]
    pub fn len_per_cell(mut self, len_per_cell: [T; D]) -> Self {
        self.len_per_cell = Some(len_per_cell);
        self
    }

    /// Set values for all dimensions
    #[must_use = "unused builder object"]
    pub fn lens(mut self, lens: [T; D]) -> Self {
        self.lens = Some(lens);
        self
    }

    /// Set origin (most bottom-left vertex) of the grid
    #[must_use = "unused builder object"]
    pub fn origin(mut self, origin: [T; D]) -> Self {
        self.origin = origin;
        self
    }

    /// Indicate whether to split cells of the grid
    ///
    /// In 2D, this will result in triangular cells.
    ///
    /// In 3D, this will result in tetrahedral cells.
    #[must_use = "unused builder object"]
    pub fn split_cells(mut self, split: bool) -> Self {
        self.split_cells = split;
        self
    }

    /// Add the attribute `A` to the attributes the created map will contain.
    ///
    /// # Usage
    ///
    /// Each attribute must be uniquely typed, i.e. a single type or struct cannot be added twice
    /// to the builder / map. This includes type aliases as these are not distinct from the
    /// compiler's perspective.
    ///
    /// If you have multiple attributes that are represented using the same data type, you may want
    /// to look into the **Newtype** pattern
    /// [here](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
    /// and [here](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
    #[must_use = "unused builder object"]
    pub fn add_attribute<A: AttributeBind + 'static>(mut self) -> Self {
        let tmp = self.map_builder.add_attribute::<A>();
        self.map_builder = tmp;
        self
    }
}

// --- parsing routine

macro_rules! check_parameters {
    ($id: ident, $msg: expr) => {
        if $id.is_sign_negative() | $id.is_zero() {
            return Err(GridBuilderError::InvalidGridParameters($msg));
        }
    };
}

impl<T: CoordsFloat> GridBuilder<2, T> {
    /// Parse provided grid parameters to provide what's used to actually generate the grid.
    #[allow(clippy::type_complexity)]
    pub(crate) fn parse_2d(
        self,
    ) -> Result<(CMapBuilder<2>, Vertex2<T>, [usize; 2], [T; 2]), GridBuilderError> {
        match (self.n_cells, self.len_per_cell, self.lens) {
            // from # cells and lengths per cell
            (Some([nx, ny]), Some([lpx, lpy]), lens) => {
                if lens.is_some() {
                    eprintln!(
                        "W: All three grid parameters were specified, total lengths will be ignored"
                    );
                }
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                Ok((
                    self.map_builder,
                    Vertex2(self.origin[0], self.origin[1]),
                    [nx, ny],
                    [lpx, lpy],
                ))
            }
            // from # cells and total lengths
            (Some([nx, ny]), None, Some([lx, ly])) => {
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                Ok((
                    self.map_builder,
                    Vertex2(self.origin[0], self.origin[1]),
                    [nx, ny],
                    [lx / T::from(nx).unwrap(), ly / T::from(ny).unwrap()],
                ))
            }
            // from lengths per cell and total lengths
            (None, Some([lpx, lpy]), Some([lx, ly])) => {
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                Ok((
                    self.map_builder,
                    Vertex2(self.origin[0], self.origin[1]),
                    [
                        (lx / lpx).ceil().to_usize().unwrap(),
                        (ly / lpy).ceil().to_usize().unwrap(),
                    ],
                    [lpx, lpy],
                ))
            }
            (_, _, _) => Err(GridBuilderError::MissingGridParameters),
        }
    }
}

impl<T: CoordsFloat> GridBuilder<3, T> {
    /// Parse provided grid parameters to provide what's used to actually generate the grid.
    #[allow(clippy::type_complexity)]
    pub(crate) fn parse_3d(
        self,
    ) -> Result<(CMapBuilder<3>, Vertex3<T>, [usize; 3], [T; 3]), GridBuilderError> {
        match (self.n_cells, self.len_per_cell, self.lens) {
            // from # cells and lengths per cell
            (Some([nx, ny, nz]), Some([lpx, lpy, lpz]), lens) => {
                if lens.is_some() {
                    eprintln!(
                        "W: All three grid parameters were specified, total lengths will be ignored"
                    );
                }
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpz, "length per z cell is null or negative");
                Ok((
                    self.map_builder,
                    Vertex3(self.origin[0], self.origin[1], self.origin[2]),
                    [nx, ny, nz],
                    [lpx, lpy, lpz],
                ))
            }
            // from # cells and total lengths
            (Some([nx, ny, nz]), None, Some([lx, ly, lz])) => {
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                #[rustfmt::skip]
                check_parameters!(lz, "grid length along z is null or negative");
                Ok((
                    self.map_builder,
                    Vertex3(self.origin[0], self.origin[1], self.origin[2]),
                    [nx, ny, nz],
                    [
                        lx / T::from(nx).unwrap(),
                        ly / T::from(ny).unwrap(),
                        lz / T::from(nz).unwrap(),
                    ],
                ))
            }
            // from lengths per cell and total lengths
            (None, Some([lpx, lpy, lpz]), Some([lx, ly, lz])) => {
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpz, "length per z cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                #[rustfmt::skip]
                check_parameters!(lz, "grid length along z is null or negative");
                Ok((
                    self.map_builder,
                    Vertex3(self.origin[0], self.origin[1], self.origin[2]),
                    [
                        (lx / lpx).ceil().to_usize().unwrap(),
                        (ly / lpy).ceil().to_usize().unwrap(),
                        (lz / lpz).ceil().to_usize().unwrap(),
                    ],
                    [lpx, lpy, lpz],
                ))
            }
            (_, _, _) => Err(GridBuilderError::MissingGridParameters),
        }
    }
}

impl<T: CoordsFloat> GridBuilder<2, T> {
    #[allow(clippy::missing_panics_doc)]
    /// Create a combinatorial map representing a 2D orthogonal grid.
    ///
    /// The map generated by this pre-definite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis:
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/user-guide/images/bg_grid.svg)
    #[must_use = "unused builder object"]
    pub fn unit_grid(n_cells_per_axis: usize) -> CMap2<T> {
        GridBuilder::default()
            .n_cells([n_cells_per_axis; 2])
            .len_per_cell([T::one(); 2])
            .build()
            .expect("E: unreachable")
    }

    #[allow(clippy::missing_panics_doc)]
    /// Create a combinatorial map representing a 2D orthogonal grid.
    ///
    /// The map generated by this pre-definite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis. Each cell is split diagonally (top left to
    /// bottom right) to form triangles:
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/user-guide/images/bg_grid_tri.svg)
    #[must_use = "unused builder object"]
    pub fn unit_triangles(n_square: usize) -> CMap2<T> {
        GridBuilder::default()
            .n_cells([n_square; 2])
            .len_per_cell([T::one(); 2])
            .split_cells(true)
            .build()
            .expect("E: unreachable")
    }

    #[allow(clippy::missing_errors_doc)]
    /// Consumes the builder and produce a combinatorial map object.
    ///
    /// This method is only available for `D == 2` or `D == 3`.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(map: CMap2<T>)` if generation was successful,
    /// - `Err(GridBuilderError)` otherwise. See [`GridBuilderError`] for possible failures.
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    pub fn build(self) -> Result<CMap2<T>, GridBuilderError> {
        let split = self.split_cells;
        self.parse_2d().map(|(builder, origin, ns, lens)| {
            if split {
                internals::build_2d_splitgrid(builder, origin, ns, lens)
            } else {
                internals::build_2d_grid(builder, origin, ns, lens)
            }
        })
    }
}

impl<T: CoordsFloat> GridBuilder<3, T> {
    #[allow(clippy::missing_panics_doc)]
    /// Create a combinatorial map representing a 3D orthogonal grid.
    ///
    /// The map generated by this pre-definite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis:
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/user-guide/images/hex.svg)
    pub fn hex_grid(n_cells_per_axis: usize, cell_length: T) -> CMap3<T> {
        GridBuilder::default()
            .n_cells([n_cells_per_axis; 3])
            .len_per_cell([cell_length; 3])
            .build()
            .expect("E: unreachable")
    }

    #[allow(clippy::missing_panics_doc)]
    /// Create a combinatorial map representing a 3D orthogonal grid.
    ///
    /// The map generated by this pre-definite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis. each hexahedral cell is cut into 5 tetrahedra;
    /// this pattern is repeated with added symmetry to produce a conformal mesh:
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/user-guide/images/tet.svg)
    #[must_use = "unused builder object"]
    pub fn tet_grid(n_cells_per_axis: usize, cell_length: T) -> CMap3<T> {
        GridBuilder::default()
            .n_cells([n_cells_per_axis; 3])
            .len_per_cell([cell_length; 3])
            .split_cells(true)
            .build()
            .expect("E: unreachable")
    }

    #[allow(clippy::missing_errors_doc)]
    /// Consumes the builder and produce a combinatorial map object.
    ///
    /// This method is only available for `D == 2` or `D == 3`.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(map: CMap3<T>)` if generation was successful,
    /// - `Err(GridBuilderError)` otherwise. See [`GridBuilderError`] for possible failures.
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    pub fn build(self) -> Result<CMap3<T>, GridBuilderError> {
        let split = self.split_cells;
        self.parse_3d().map(|(builder, origin, ns, lens)| {
            if split {
                internals::build_3d_tetgrid(builder, origin, ns, lens)
            } else {
                internals::build_3d_grid(builder, origin, ns, lens)
            }
        })
    }
}
