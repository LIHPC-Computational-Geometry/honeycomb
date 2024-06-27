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

pub fn load_geometry<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path> + std::fmt::Debug,
) -> Geometry2<T> {
    todo!()
}

pub struct Geometry2<T: CoordsFloat> {
    vertices: Vec<Vertex2<T>>,
    segments: Vec<Segment>,
    poi: Vec<usize>,
}

impl<T: CoordsFloat> Geometry2<T> {
    pub fn bbox(&self) -> BBox2<T> {
        todo!()
    }
}

impl<T: CoordsFloat> From<Vtk> for Geometry2<T> {
    fn from(value: Vtk) -> Self {
        todo!()
    }
}

pub struct Segment(usize, usize);
