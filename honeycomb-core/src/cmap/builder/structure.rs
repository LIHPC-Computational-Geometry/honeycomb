use std::fs::File;
use std::io::Read;

use thiserror::Error;
use vtkio::Vtk;

use crate::attributes::{AttrStorageManager, AttributeBind};
use crate::cmap::{CMap2, CMap3};
use crate::geometry::CoordsFloat;

use super::io::CMapFile;

/// # Builder-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when using the builder
/// structure.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum BuilderError {
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
/// use honeycomb_core::cmap::{CMap2, CMap3, CMapBuilder};
///
/// let builder_2d = CMapBuilder::<2>::from_n_darts(10);
/// let map_2d: CMap2<f64> = builder_2d.build()?;
/// assert_eq!(map_2d.n_darts(), 11); // 10 + null dart = 11
///
/// let builder_3d = CMapBuilder::<3>::from_n_darts(10);
/// let map_3d: CMap3<f64> = builder_3d.build()?;
/// assert_eq!(map_3d.n_darts(), 11); // 10 + null dart = 11
/// # Ok(())
/// # }
/// ```
pub struct CMapBuilder<const D: usize> {
    builder_kind: BuilderType,
    attributes: AttrStorageManager,
    enable_vid_cache: bool,
}

enum BuilderType {
    CMap(CMapFile),
    FreeDarts(usize),
    Vtk(Vtk),
}

#[doc(hidden)]
pub trait Builder<T: CoordsFloat> {
    type MapType;
    fn build(self) -> Result<Self::MapType, BuilderError>;
}

impl<T: CoordsFloat> Builder<T> for CMapBuilder<2> {
    type MapType = CMap2<T>;

    fn build(self) -> Result<Self::MapType, BuilderError> {
        match self.builder_kind {
            BuilderType::CMap(cfile) => super::io::build_2d_from_cmap_file(cfile, self.attributes),
            BuilderType::FreeDarts(n_darts) => Ok(CMap2::new_with_undefined_attributes(
                n_darts,
                self.attributes,
            )),
            BuilderType::Vtk(vfile) => super::io::build_2d_from_vtk(vfile, self.attributes),
        }
    }
}

impl<T: CoordsFloat> Builder<T> for CMapBuilder<3> {
    type MapType = CMap3<T>;

    fn build(self) -> Result<Self::MapType, BuilderError> {
        match self.builder_kind {
            BuilderType::CMap(cfile) => {
                super::io::build_3d_from_cmap_file(cfile, self.attributes, self.enable_vid_cache)
            }
            BuilderType::FreeDarts(n_darts) => Ok(CMap3::from_data(
                n_darts,
                self.attributes,
                self.enable_vid_cache,
            )),
            BuilderType::Vtk(_vfile) => unimplemented!(),
        }
    }
}
/// # Regular methods
impl<const D: usize> CMapBuilder<D> {
    /// Create a builder structure for a map with a set number of darts and the attribute set of
    /// another builder.
    #[must_use = "unused builder object"]
    pub fn from_n_darts_and_attributes(n_darts: usize, other: Self) -> Self {
        Self {
            builder_kind: BuilderType::FreeDarts(n_darts),
            attributes: other.attributes,
            enable_vid_cache: false,
        }
    }

    /// Create a builder structure for a map with a set number of darts.
    #[must_use = "unused builder object"]
    pub fn from_n_darts(n_darts: usize) -> Self {
        Self {
            builder_kind: BuilderType::FreeDarts(n_darts),
            attributes: AttrStorageManager::default(),
            enable_vid_cache: false,
        }
    }

    /// Create a builder structure from a `cmap` file.
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
            enable_vid_cache: false,
        }
    }

    /// Create a builder structure from a VTK file.
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
            enable_vid_cache: false,
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

    /// Enable usage of an internal vertex ID cache.
    ///
    /// By default, vertex IDs are recomputed at each call of the `vertex_id(_tx)` methods. By
    /// enabling this cache, vertex IDs associated to darts are instead stored in a dedicated
    /// collection, and updated on (un)sews. This can be useful for algorithm which frequently
    /// use this data.
    #[must_use = "unused builder object"]
    pub fn enable_vertex_id_cache(mut self, enable: bool) -> Self {
        self.enable_vid_cache = enable;
        self
    }

    #[allow(clippy::missing_errors_doc)]
    /// Consumes the builder and produce a combinatorial map object.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(map: _)` if generation was successful,
    /// - `Err(BuilderError)` otherwise. See [`BuilderError`] for possible failures.
    ///
    /// Depending on the dimension `D` associated with this structure, the map will either be a
    /// `CMap2` or `CMap3`. If `D` isn't 2 or 3, this method will not be available as it uses a
    /// trait not implemented for other values of `D`. This is necessary to handle the multiple
    /// return types as Rust is slightly lacking in terms of comptime capabilities.
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    #[allow(private_interfaces, private_bounds)]
    pub fn build<T: CoordsFloat>(self) -> Result<<Self as Builder<T>>::MapType, BuilderError>
    where
        Self: Builder<T>,
    {
        Builder::build(self)
    }
}
