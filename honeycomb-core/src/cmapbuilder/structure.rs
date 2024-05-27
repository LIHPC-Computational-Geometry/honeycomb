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
    pub fn build2(self) -> Result<CMap2<T>, CMapError> {
        todo!()
    }

    pub fn build3(self) {
        unimplemented!("E: 3-maps are not yet implemented")
    }
}

// --- predefinite setups

impl<T: CoordsFloat> CMapBuilder<T> {
    pub fn unit_grid(n_square: usize) -> Self {
        todo!()
    }

    pub fn unit_split_grid(n_square: usize) -> Self {
        todo!()
    }
}
