//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, Vertex2};
use num::Zero;
use vtkio::model::{DataSet, Vtk};
use vtkio::IOBuffer;

// ------ CONTENT

// --- macros

macro_rules! build_vertices {
    ($v: ident) => {{
        assert!(
            ($v.len() % 3).is_zero(),
            "failed to fetch vertex data - the point list contains an incomplete tuple"
        );
        $v.chunks_exact(3)
            .map(|slice| {
                // WE IGNORE Z values
                let &[x, y, _] = slice else { panic!() };
                Vertex2::from((T::from(x).unwrap(), T::from(y).unwrap()))
            })
            .collect()
    }};
}

// --- implementations

impl<T: CoordsFloat> From<Vtk> for CMap2<T> {
    fn from(value: Vtk) -> Self {
        match value.data {
            DataSet::ImageData { .. }
            | DataSet::StructuredGrid { .. }
            | DataSet::RectilinearGrid { .. }
            | DataSet::PolyData { .. }
            | DataSet::Field { .. } => {}
            DataSet::UnstructuredGrid { pieces, .. } => pieces.iter().for_each(|piece| {
                // assume inline data
                let tmp = piece
                    .load_piece_data(None)
                    .expect("failed to load piece data");

                // build vertex list
                // since we're expecting coordinates, we'll assume floating type
                // we're also converting directly to our vertex type since we're building a 2-map
                let vertices: Vec<Vertex2<T>> = match tmp.points {
                    IOBuffer::F64(v) => build_vertices!(v),
                    IOBuffer::F32(v) => build_vertices!(v),
                    _ => unimplemented!(),
                };
            }),
        }
        todo!()
    }
}

impl<T: CoordsFloat> From<CMap2<T>> for Vtk {
    fn from(value: CMap2<T>) -> Self {
        todo!()
    }
}
