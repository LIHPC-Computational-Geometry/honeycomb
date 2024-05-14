//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, DartIdentifier, Vertex2, VertexIdentifier};
use num::Zero;
use std::collections::BTreeMap;
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

impl<T: CoordsFloat> CMap2<T> {
    pub fn from_vtk_file(file_path: &str) -> Self {
        use std::path::PathBuf;
        let file_path = PathBuf::from(file_path);
        let file = Vtk::import(&file_path)
            .unwrap_or_else(|_| panic!("Failed to load file: {file_path:?}"));
        build_cmap_from_vtk(file)
    }
}

// --- internals

fn build_cmap_from_vtk<T: CoordsFloat>(value: Vtk) -> CMap2<T> {
    let mut cmap: CMap2<T> = CMap2::new(0);
    let mut sew_buffer: BTreeMap<(usize, usize), DartIdentifier> = BTreeMap::new();
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
                    vertices: verts,
                } => {
                    // check basic stuff
                    assert_eq!(
                        num_cells as usize,
                        types.len(),
                        "failed to build cells - inconsistent number of cell between CELLS and CELL_TYPES"
                    );

                    // build a collection of vertex lists corresponding of each cell
                    let mut cell_components: Vec<Vec<usize>> = Vec::new();
                    let mut take_next = 0;
                    verts.iter().for_each(|vertex_id| if take_next.is_zero() {
                        // making it usize since it's a counter
                        take_next = *vertex_id as usize;
                        cell_components.push(Vec::with_capacity(take_next));
                    } else {
                        cell_components.last_mut().unwrap().push(*vertex_id as usize);
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
                            // check validity
                            assert_eq!(vids.len(), 3, "failed to build cell - `Triangle` has {} instead of 3 vertices", vids.len());
                            // build the triangle
                            let d0 = cmap.add_free_darts(3);
                            let (d1, d2) = (d0+1, d0+2);
                            cmap.insert_vertex(d0 as VertexIdentifier, vertices[vids[0]]);
                            cmap.insert_vertex(d1 as VertexIdentifier, vertices[vids[1]]);
                            cmap.insert_vertex(d2 as VertexIdentifier, vertices[vids[2]]);
                            cmap.one_link(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                            cmap.one_link(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                            cmap.one_link(d2, d0); // edge d2 links vertices vids[2] & vids[0]
                            // record a trace of the built cell for future 2-sew
                            sew_buffer.insert((vids[0], vids[1]), d0);
                            sew_buffer.insert((vids[1], vids[2]), d1);
                            sew_buffer.insert((vids[2], vids[0]), d2);
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
                    todo!("XML file format is not currently supported")
                }
            }
        }),
    }
    while let Some(((id0, id1), dart_id0)) = sew_buffer.pop_first() {
        if let Some(dart_id1) = sew_buffer.remove(&(id1, id0)) {
            cmap.two_sew(dart_id0, dart_id1);
        }
    }

    cmap
}
