use std::fs::File;
use std::io::Read;

use thiserror::Error;
use vtkio::Vtk;

use crate::attributes::{AttrStorageManager, AttributeBind};
use crate::cmap::{CMap2, GridDescriptor};
use crate::geometry::CoordsFloat;

use super::io::CMapFile;

/// # Builder-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when using the builder
/// structure.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum BuilderError {
    // grid-related variants
    /// One or multiple of the specified grid characteristics are invalid.
    #[error("invalid grid parameters - {0}")]
    InvalidGridParameters(&'static str),
    /// The builder is missing one or multiple parameters to generate the grid.
    #[error("insufficient parameters - please specifiy at least 2")]
    MissingGridParameters,

    // custom format variants
    /// A value could not be parsed.
    #[error("error parsing a value in one of the cmap file section - {0}")]
    BadValue(&'static str),
    /// The meta section of the file is incorrect.
    #[error("error parsing the cmap file meta section - {0}")]
    BadMetaData(&'static str),
    /// The file contains a duplicated section.
    #[error("duplicated section in cmap file - {0}")]
    DuplicatedSection(String),
    /// The file contains contradicting data.
    #[error("inconsistent data - {0}")]
    InconsistentData(&'static str),
    /// A required section is missing from the file.
    #[error("required section missing in cmap file - {0}")]
    MissingSection(&'static str),
    /// The file contains an unrecognized section header.
    #[error("unknown header in cmap file - {0}")]
    UnknownHeader(String),

    // vtk-related variants
    /// Specified VTK file contains inconsistent data.
    #[error("invalid/corrupted data in the vtk file - {0}")]
    BadVtkData(&'static str),
    /// Specified VTK file contains unsupported data.
    #[error("unsupported data in the vtk file - {0}")]
    UnsupportedVtkData(&'static str),
}

/// # Combinatorial map builder structure
///
/// ## Example
///
/// ```rust
/// # use honeycomb_core::cmap::BuilderError;
/// # fn main() -> Result<(), BuilderError> {
/// use honeycomb_core::cmap::{CMap2, CMapBuilder};
///
/// let builder = CMapBuilder::from_n_darts(10);
/// let map: CMap2<f64> = builder.build()?;
///
/// assert_eq!(map.n_darts(), 11); // 10 + null dart = 11
///
/// # Ok(())
/// # }
/// ```
pub struct CMapBuilder<T>
where
    T: CoordsFloat,
{
    builder_kind: BuilderType<T>,
    attributes: AttrStorageManager,
}

enum BuilderType<T: CoordsFloat> {
    CMap(CMapFile),
    FreeDarts(usize),
    Grid(GridDescriptor<T>),
    Vtk(Vtk),
}

/// # Regular methods
impl<T: CoordsFloat> CMapBuilder<T> {
    /// Set the number of dart that the created map will contain.
    #[must_use = "unused builder object"]
    pub fn from_n_darts(n_darts: usize) -> Self {
        Self {
            builder_kind: BuilderType::FreeDarts(n_darts),
            attributes: AttrStorageManager::default(),
        }
    }

    /// Set the [`GridDescriptor`] that will be used when building the map.
    #[must_use = "unused builder object"]
    pub fn from_grid_descriptor(grid_descriptor: GridDescriptor<T>) -> Self {
        Self {
            builder_kind: BuilderType::Grid(grid_descriptor),
            attributes: AttrStorageManager::default(),
        }
    }

    /// Set the `cmap` file that will be used when building the map.
    ///
    /// # Panics
    ///
    /// This function may panic if the file cannot be loaded, or basic section parsing fails.
    #[must_use = "unused builder object"]
    pub fn from_cmap_file(file_path: impl AsRef<std::path::Path> + std::fmt::Debug) -> Self {
        let mut f = File::open(file_path).expect("E: could not open specified file");
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .expect("E: could not read content from file");
        let cmap_file = CMapFile::try_from(buf).unwrap();

        Self {
            builder_kind: BuilderType::CMap(cmap_file),
            attributes: AttrStorageManager::default(),
        }
    }

    /// Set the VTK file that will be used when building the map.
    ///
    /// # Panics
    ///
    /// This function may panic if the file cannot be loaded.
    #[must_use = "unused builder object"]
    pub fn from_vtk_file(file_path: impl AsRef<std::path::Path> + std::fmt::Debug) -> Self {
        let vtk_file =
            Vtk::import(file_path).unwrap_or_else(|e| panic!("E: failed to load file: {e:?}"));

        Self {
            builder_kind: BuilderType::Vtk(vtk_file),
            attributes: AttrStorageManager::default(),
        }
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
        self.attributes.add_storage::<A>(1);
        self
    }

    #[allow(clippy::missing_errors_doc)]
    /// Consumes the builder and produce a [`CMap2`] object.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(map: CMap2)` if generation was successful,
    /// - `Err(BuilderError)` otherwise. See [`BuilderError`] for possible failures.
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    pub fn build(self) -> Result<CMap2<T>, BuilderError> {
        match self.builder_kind {
            BuilderType::CMap(cfile) => super::io::build_2d_from_cmap_file(cfile, self.attributes),
            BuilderType::FreeDarts(n_darts) => Ok(CMap2::new_with_undefined_attributes(
                n_darts,
                self.attributes,
            )),
            BuilderType::Grid(gridb) => {
                let split = gridb.split_quads;
                gridb.parse_2d().map(|(origin, ns, lens)| {
                    if split {
                        super::grid::build_2d_splitgrid(origin, ns, lens, self.attributes)
                    } else {
                        super::grid::build_2d_grid(origin, ns, lens, self.attributes)
                    }
                })
            }
            BuilderType::Vtk(vfile) => super::io::build_2d_from_vtk(vfile, self.attributes),
        }
    }
}

/// # Pre-definite structures
impl<T: CoordsFloat> CMapBuilder<T> {
    /// Create a [`CMapBuilder`] with a predefinite [`GridDescriptor`] value.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Number of cells along each axis.
    ///
    /// # Return
    ///
    /// This function return a builder structure with pre-definite parameters set to generate
    /// a specific map.
    ///
    /// The map generated by this pre-definite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis:
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/user-guide/images/bg_grid.svg)
    #[must_use = "unused builder object"]
    pub fn unit_grid(n_square: usize) -> Self {
        Self {
            builder_kind: BuilderType::Grid(
                GridDescriptor::default()
                    .n_cells([n_square; 3])
                    .len_per_cell([T::one(); 3]),
            ),
            attributes: AttrStorageManager::default(),
        }
    }

    /// Create a [`CMapBuilder`] with a predefinite [`GridDescriptor`] value.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Number of cells along each axis.
    ///
    /// # Return
    ///
    /// This function return a builder structure with pre-definite parameters set to generate
    /// a specific map.
    ///
    /// The map generated by this pre-definite value corresponds to an orthogonal mesh, with an
    /// equal number of cells along each axis. Each cell will be split across their diagonal (top
    /// left to bottom right) to form triangles:
    ///
    /// ![`CMAP2_GRID`](https://lihpc-computational-geometry.github.io/honeycomb/user-guide/images/bg_grid_tri.svg)
    #[must_use = "unused builder object"]
    pub fn unit_triangles(n_square: usize) -> Self {
        Self {
            builder_kind: BuilderType::Grid(
                GridDescriptor::default()
                    .n_cells([n_square; 3])
                    .len_per_cell([T::one(); 3])
                    .split_quads(true),
            ),
            attributes: AttrStorageManager::default(),
        }
    }
}
