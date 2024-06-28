//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::BBox2;
use honeycomb_core::{CoordsFloat, Vertex2};
use vtkio::Vtk;

// ------ CONTENT

/// Post-processing clamp operation.
#[derive(Default)]
pub enum Clamp {
    /// Clamp inner & outer cells, leaving only boundaries of the geometry.
    All,
    /// Clamp inner cells.
    Inner,
    /// Clamp outer cells.
    Outer,
    /// Do nothing. Default value.
    #[default]
    None,
}

/// Build a [Geometry2] object from a VTK file.
pub fn load_geometry<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path> + std::fmt::Debug,
) -> Geometry2<T> {
    todo!()
}

/// Geometry representation structure.
pub struct Geometry2<T: CoordsFloat> {
    /// Vertices of the geometry.
    vertices: Vec<Vertex2<T>>,
    /// Edges / segments making up the geometry.
    segments: Vec<Segment>,
    /// Points of interest, i.e. points to insert unconditionally in the future map / mesh.
    poi: Vec<usize>,
}

impl<T: CoordsFloat> Geometry2<T> {
    /// Return the bounding box of the geometry.
    pub fn bbox(&self) -> BBox2<T> {
        todo!()
    }
}

impl<T: CoordsFloat> From<Vtk> for Geometry2<T> {
    fn from(value: Vtk) -> Self {
        todo!()
    }
}

/// Segment modelling structure.
///
/// Inner values correspond to vertex indices, order matters.
pub struct Segment(usize, usize);
