//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

#[cfg(feature = "utils")]
use crate::GridDescriptor;
use crate::{AttrStorageManager, AttributeBind, CMap2, CoordsFloat};
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
    // grid-related variants
    /// One or multiple of the specified grid characteristics are invalid.
    InvalidGridParameters(&'static str),
    /// The builder is missing one or multiple parameters to generate the grid.
    MissingGridParameters(&'static str),
    // vtk-related variants
    /// Specified VTK file contains inconsistent data.
    InvalidVtkFile(&'static str),
    /// Specified VTK file could not be found.
    MissingVtkFile(&'static str),
    /// Specified VTK file contains unsupported data.
    UnsupportedVtkData(&'static str),
}

// --- main struct

/// Combinatorial map builder structure.
///
/// #
///
/// # Example
///
/// ```rust
/// # use honeycomb_core::BuilderError;
/// # fn main() -> Result<(), BuilderError> {
/// use honeycomb_core::{CMap2, CMapBuilder};
///
/// let builder = CMapBuilder::default().n_darts(10);
/// let map: CMap2<f64> = builder.build()?;
///
/// assert_eq!(map.n_darts(), 11); // 10 + null dart = 11
///
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct CMapBuilder<T>
where
    T: CoordsFloat,
{
    #[cfg(feature = "io")]
    pub(super) vtk_file: Option<Vtk>,
    #[cfg(feature = "utils")]
    pub(super) grid_descriptor: Option<GridDescriptor<T>>,
    pub(super) attributes: AttrStorageManager,
    pub(super) n_darts: usize,
    pub(super) coordstype: std::marker::PhantomData<T>,
}

// --- setters

impl<T: CoordsFloat> CMapBuilder<T> {
    /// Set the number of dart that the created map will contain.
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn n_darts(mut self, n_darts: usize) -> Self {
        self.n_darts = n_darts;
        self
    }

    /// Add the specified attribute that the created map will contain.
    ///
    /// Each attribute must be uniquely typed, i.e. a single type or struct cannot be added twice
    /// to the builder / map. This includes type aliases as these are not distinct from the
    /// compiler's perspective.
    ///
    /// If you have multiple attributes that are represented using the same data type, you may want
    /// to look into the **Newtype** pattern
    /// [here](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
    /// and [here](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn add_attribute<A: AttributeBind + 'static>(mut self) -> Self {
        if self.attributes.add_storage::<A>(0).is_err() {
            println!("W: attribute already added to the builder - continuing...");
        }
        self
    }
}

// --- build methods

impl<T: CoordsFloat> CMapBuilder<T> {
    #[allow(clippy::missing_errors_doc)]
    /// Consumes the builder and produce a [`CMap2`] object.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(map: CMap2)` -- Map generation was successful.
    /// - `Err(BuilderError::MissingParameters)` -- The specified grid parameters are insufficient
    ///   to build a map from it.
    /// - `Err(BuilderError::InvalidParameters)` -- The specified grid parameters contain at least
    ///   one invalid value (e.g. a negative length).
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    ///
    /// # Example
    ///
    /// See [`CMapBuilder`] example.
    ///
    pub fn build(mut self) -> Result<CMap2<T>, BuilderError> {
        #[cfg(feature = "io")]
        if let Some(vfile) = self.vtk_file {
            // build from vtk
            // this routine should return a Result instead of the map directly
            return super::io::build_2d_from_vtk(vfile, self.attributes);
        }
        #[cfg(feature = "utils")]
        if let Some(gridb) = self.grid_descriptor {
            // build from grid descriptor
            return if gridb.split_quads {
                gridb.parse_2d().map(|(ns, lens)| {
                    super::grid::building_routines::build_2d_splitgrid(ns, lens, self.attributes)
                })
            } else {
                gridb.parse_2d().map(|(ns, lens)| {
                    super::grid::building_routines::build_2d_grid(ns, lens, self.attributes)
                })
            };
        }
        self.attributes.extend_storages(self.n_darts); // todo: add it to the map
        Ok(CMap2::new(self.n_darts))
    }
}
