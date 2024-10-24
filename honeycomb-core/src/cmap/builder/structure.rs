//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

#[cfg(feature = "utils")]
use super::GridDescriptor;
use crate::prelude::{AttributeBind, CMap2};
use crate::{attributes::AttrStorageManager, geometry::CoordsFloat};
use thiserror::Error;
#[cfg(feature = "io")]
use vtkio::Vtk;

// ------ CONTENT

// --- common error enum

/// Builder-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when using a builder
/// structure.
#[derive(Error, Debug)]
pub enum BuilderError {
    // grid-related variants
    /// One or multiple of the specified grid characteristics are invalid.
    #[error("invalid grid parameters - {0}")]
    InvalidGridParameters(&'static str),
    /// The builder is missing one or multiple parameters to generate the grid.
    #[error("insufficient parameters - please specifiy at least 2")]
    MissingGridParameters,
    // vtk-related variants
    /// Specified VTK file contains inconsistent data.
    #[error("invalid/corrupted data in the vtk file - {0}")]
    BadVtkData(&'static str),
    /// Specified VTK file contains unsupported data.
    #[error("unsupported data in the vtk file - {0}")]
    UnsupportedVtkData(&'static str),
}

// --- main struct

/// Combinatorial map builder structure.
///
/// # Example
///
/// ```rust
/// # use honeycomb_core::prelude::BuilderError;
/// # fn main() -> Result<(), BuilderError> {
/// use honeycomb_core::prelude::{CMap2, CMapBuilder};
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
        self.attributes.add_storage::<A>(self.n_darts);
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
    /// - `Err(BuilderError)` -- There was an error during construction. See [`BuilderError`] for
    ///   details.
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    ///
    /// # Example
    ///
    /// See [`CMapBuilder`] example.
    pub fn build(self) -> Result<CMap2<T>, BuilderError> {
        #[cfg(feature = "io")]
        if let Some(vfile) = self.vtk_file {
            // build from vtk
            // this routine should return a Result instead of the map directly
            return super::io::build_2d_from_vtk(vfile, self.attributes);
        }
        #[cfg(feature = "utils")]
        if let Some(gridb) = self.grid_descriptor {
            // build from grid descriptor
            let split = gridb.split_quads;
            return gridb.parse_2d().map(|(origin, ns, lens)| {
                if split {
                    super::grid::building_routines::build_2d_splitgrid(
                        origin,
                        ns,
                        lens,
                        self.attributes,
                    )
                } else {
                    super::grid::building_routines::build_2d_grid(origin, ns, lens, self.attributes)
                }
            });
        }
        Ok(CMap2::new_with_undefined_attributes(
            self.n_darts,
            self.attributes,
        ))
    }
}
