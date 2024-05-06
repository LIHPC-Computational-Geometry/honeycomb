//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use vtkio::xml::VTKFile;

use crate::{CMap2, CoordsFloat};

// ------ CONTENT

impl<T: CoordsFloat> From<VTKFile> for CMap2<T> {
    fn from(value: VTKFile) -> Self {
        todo!()
    }
}

impl<T: CoordsFloat> From<CMap2<T>> for VTKFile {
    fn from(value: CMap2<T>) -> Self {
        todo!()
    }
}
