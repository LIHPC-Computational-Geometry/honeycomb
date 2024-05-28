//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

#[cfg(feature = "utils")]
use crate::utils::GridBuilder;
use crate::{CMap2, CMapError, CoordsFloat};
#[cfg(feature = "io")]
use vtkio::Vtk;

// ------ CONTENT

// --- common error enum

/// Builder-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when using a builder
/// structure.
#[derive(Debug)]
pub enum BuilderError {
    /// The builder is missing one or multiple parameters in order to proceed with the requested
    /// operation.
    MissingParameters(&'static str),
    /// One or multiple of the builder's fields are invalid.
    InvalidParameters(&'static str),
}

// --- main struct

#[derive(Default)]
pub struct CMapBuilder<T>
where
    T: CoordsFloat,
{
    #[cfg(feature = "io")]
    vtk_file: Option<Vtk>,
    #[cfg(feature = "utils")]
    grid_builder: Option<GridBuilder<T>>,
    n_darts: usize,
    coordstype: std::marker::PhantomData<T>,
}

// --- setters

impl<T: CoordsFloat> CMapBuilder<T> {
    pub fn n_darts(mut self, n_darts: usize) -> Self {
        self.n_darts = n_darts;
        self
    }

    #[cfg(feature = "io")]
    pub fn using_vtk_file(
        mut self,
        file_path: impl AsRef<std::path::Path> + std::fmt::Debug,
    ) -> Self {
        let vtk_file =
            Vtk::import(file_path).unwrap_or_else(|e| panic!("E: failed to load file: {e:?}"));
        self.vtk_file = Some(vtk_file);
        self
    }

    #[cfg(feature = "utils")]
    pub fn using_grid_builder(mut self, grid_builder: GridBuilder<T>) -> Self {
        self.grid_builder = Some(grid_builder);
        self
    }
}

// --- build methods

impl<T: CoordsFloat> CMapBuilder<T> {
    pub fn build2(self) -> Result<CMap2<T>, BuilderError> {
        #[cfg(feature = "io")]
        if let Some(vfile) = self.vtk_file {
            // build from vtk
            todo!()
        }
        #[cfg(feature = "utils")]
        if let Some(gridb) = self.grid_builder {
            // build from grid descriptor
            return if gridb.split_quads {
                gridb
                    .parse()
                    .map(|(ns, lens)| super::grid::build2_splitgrid(ns, lens))
            } else {
                gridb
                    .parse()
                    .map(|(ns, lens)| super::grid::build2_grid(ns, lens))
            };
        }
        Ok(CMap2::new(self.n_darts))
    }

    pub fn build3(self) {
        unimplemented!("E: 3-maps are not yet implemented")
    }
}

// --- predefinite grid setups

#[cfg(feature = "utils")]
impl<T: CoordsFloat> CMapBuilder<T> {
    pub fn unit_grid(n_square: usize) -> Self {
        let gridb = GridBuilder::default()
            .n_cells([n_square; 3])
            .len_per_cell([T::one(); 3]);
        CMapBuilder::default().using_grid_builder(gridb)
    }

    pub fn unit_split_grid(n_square: usize) -> Self {
        let gridb = GridBuilder::default()
            .n_cells([n_square; 3])
            .len_per_cell([T::one(); 3])
            .split_quads(true);
        CMapBuilder::default().using_grid_builder(gridb)
    }
}
