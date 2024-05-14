//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, Vertex2, VertexIdentifier};
use num::Zero;
use vtkio::model::{CellType, Cells, DataSet, VertexNumbers, Vtk};
use vtkio::IOBuffer;

// ------ CONTENT

// --- macros

macro_rules! build_vertices {
    ($v: ident) => {{
        assert!(
            ($v.len() % 3).is_zero(),
            "failed to build vertices list - the point list contains an incomplete tuple"
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

                let Cells { cell_verts, types } = tmp.cells;
                match cell_verts {
                    VertexNumbers::Legacy {
                        num_cells,
                        vertices,
                    } => {
                        // check basic stuff
                        assert_eq!(
                            num_cells as usize,
                            types.len(),
                            "failed to build cells - inconsistent number of cell between CELLS and CELL_TYPES"
                        );

                        // build a collection of vertex lists correspondign of each cell
                        let mut cell_components: Vec<Vec<VertexIdentifier>> = Vec::new();
                        let mut take_next = 0;
                        vertices.iter().for_each(|vertex_id| if take_next.is_zero() {
                            // making it usize since it's a counter
                            take_next = *vertex_id as usize;
                            cell_components.push(Vec::with_capacity(take_next));
                        } else {
                            cell_components.last_mut().unwrap().push(*vertex_id as VertexIdentifier);
                            take_next -= 1;
                        });
                        assert_eq!(num_cells as usize, cell_components.len());

                        types.iter().zip(cell_components.iter()).for_each(|(cell_type, vids)| match cell_type {
                            CellType::Vertex => {
                                assert_eq!(vids.len(), 1, "failed to build cell - `Vertex` has {} instead of 1 vertex", vids.len());
                            }
                            CellType::PolyVertex => {}
                            CellType::Line => {
                                assert_eq!(vids.len(), 2, "failed to build cell - `Line` has {} instead of 2 vertices", vids.len());
                            }
                            CellType::PolyLine => {}
                            CellType::Triangle => {
                                assert_eq!(vids.len(), 3, "failed to build cell - `Triangle` has {} instead of 3 vertices", vids.len());
                            }
                            CellType::TriangleStrip => unimplemented!(
                                "failed to build cell - `TriangleStrip` cell type is not supported because of orientation requirements"
                            ),
                            CellType::Polygon => {}
                            CellType::Pixel => unimplemented!(
                                "failed to build cell - `Pixel` cell type is not supported because of orientation requirements"
                            ),
                            CellType::Quad => {
                                assert_eq!(vids.len(), 4,  "failed to build cell - `Quad` has {} instead of 4 vertices", vids.len());
                            }
                            c => unimplemented!(
                                "failed to build cell - {c:#?} is not supported in 2-maps"
                            ),
                        });
                    }
                    VertexNumbers::XML { .. } => {
                        todo!()
                    }
                }
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
