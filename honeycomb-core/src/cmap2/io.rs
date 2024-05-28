//! Input/Output features implementation
//!
//! The support for I/O is currently very restricted since this is not the focus of this project.
//! Maps can be built from and serialized to VTK legacy files (both binary and ASCII). The
//! `DATASET` of the VTK file should be `UNSTRUCTURED_GRID`, and only a given set of `CELL_TYPES`
//! are supported, because of orientation and dimension restriction.

// ------ IMPORTS

use crate::{
    CMap2, Coords2, CoordsFloat, DartIdentifier, Orbit2, OrbitPolicy, Vertex2, VertexIdentifier,
    NULL_DART_ID,
};

use std::{any::TypeId, collections::BTreeMap};

use num::Zero;
use vtkio::{
    model::{
        ByteOrder, CellType, DataSet, Piece, UnstructuredGridPiece, Version, VertexNumbers, Vtk,
    },
    IOBuffer,
};

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

impl<T: CoordsFloat + 'static> CMap2<T> {
    /// Generate a legacy VTK file from the map.
    ///
    /// # Panics
    ///
    /// This function may panic if the internal writing routine fails, i.e.:
    ///     - Vertex coordinates cannot be cast to `f32` or `f64`
    ///     - A vertex cannot be found
    pub fn to_vtk_binary(&self, writer: impl std::io::Write) {
        // build a Vtk structure
        let vtk_struct = Vtk {
            version: Version { major: 2, minor: 0 },
            title: "cmap".to_string(),
            byte_order: ByteOrder::BigEndian,
            data: DataSet::UnstructuredGrid {
                meta: None,
                pieces: vec![Piece::Inline(Box::new(build_unstructured_piece(self)))],
            },
            file_path: None,
        };

        // write data to the created file
        vtk_struct
            .write_legacy(writer)
            .expect("Could not write data to writer");
    }

    /// Generate a legacy VTK file from the map.
    ///
    /// # Panics
    ///
    /// This function may panic if the internal writing routine fails, i.e.:
    ///     - Vertex coordinates cannot be cast to `f32` or `f64`
    ///     - A vertex cannot be found
    pub fn to_vtk_ascii(&self, writer: impl std::fmt::Write) {
        // build a Vtk structure
        let vtk_struct = Vtk {
            version: Version { major: 2, minor: 0 },
            title: "cmap".to_string(),
            byte_order: ByteOrder::BigEndian,
            data: DataSet::UnstructuredGrid {
                meta: None,
                pieces: vec![Piece::Inline(Box::new(build_unstructured_piece(self)))],
            },
            file_path: None,
        };

        // write data to the created file
        vtk_struct
            .write_legacy_ascii(writer)
            .expect("Could not write data to writer");
    }
}

// --- internals

#[allow(clippy::too_many_lines)]
/// Internal building routine for [`CMap2::from_vtk_file`].
///
/// This is marked as `pub(super)` for testing purposes.
pub(super) fn build_cmap_from_vtk<T: CoordsFloat>(value: Vtk) -> CMap2<T> {
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

            let vtkio::model::Cells { cell_verts, types } = tmp.cells;
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
                            // silent ignore
                        }
                        CellType::PolyVertex => unimplemented!(
                            "failed to build cell - `PolyVertex` cell type is not supported because for consistency"
                        ),
                        CellType::Line => {
                            assert_eq!(vids.len(), 2, "failed to build cell - `Line` has {} instead of 2 vertices", vids.len());
                            // silent ignore
                        }
                        CellType::PolyLine => unimplemented!(
                            "failed to build cell - `PolyLine` cell type is not supported because for consistency"
                        ),
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
                        CellType::Polygon => {
                            // FIXME: NOT TESTED
                            // operation order should still work, but it would be nice to have 
                            // an heterogeneous mesh to test on
                            let n_vertices = vids.len();
                            let d0 = cmap.add_free_darts(n_vertices);
                            (0..n_vertices ).for_each(|i| {
                                let di = d0 + i as DartIdentifier;
                                let dip1 = if i==n_vertices-1 {
                                    d0
                                } else {
                                    di +1
                                };
                                cmap.insert_vertex(di as VertexIdentifier, vertices[vids[i]]);
                                cmap.one_link(di, dip1);
                                sew_buffer.insert((vids[i], vids[(i + 1) % n_vertices]), di);
                            });
                        }
                        CellType::Pixel => unimplemented!(
                            "failed to build cell - `Pixel` cell type is not supported because of orientation requirements"
                        ),
                        CellType::Quad => {
                            assert_eq!(vids.len(), 4,  "failed to build cell - `Quad` has {} instead of 4 vertices", vids.len());
                            // build the quad
                            let d0 = cmap.add_free_darts(4);
                            let (d1, d2, d3) = (d0+1, d0+2, d0+3);
                            cmap.insert_vertex(d0 as VertexIdentifier, vertices[vids[0]]);
                            cmap.insert_vertex(d1 as VertexIdentifier, vertices[vids[1]]);
                            cmap.insert_vertex(d2 as VertexIdentifier, vertices[vids[2]]);
                            cmap.insert_vertex(d3 as VertexIdentifier, vertices[vids[3]]);
                            cmap.one_link(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                            cmap.one_link(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                            cmap.one_link(d2, d3); // edge d2 links vertices vids[2] & vids[3]
                            cmap.one_link(d3, d0); // edge d3 links vertices vids[3] & vids[0]
                            // record a trace of the built cell for future 2-sew
                            sew_buffer.insert((vids[0], vids[1]), d0);
                            sew_buffer.insert((vids[1], vids[2]), d1);
                            sew_buffer.insert((vids[2], vids[3]), d2);
                            sew_buffer.insert((vids[3], vids[0]), d3);
                        }
                        c => unimplemented!(
                            "failed to build cell - {c:#?} is not supported in 2-maps"
                        ),
                    });
                }
                VertexNumbers::XML { .. } => {
                    unimplemented!("XML file format is not currently supported")
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

/// Internal building routine for [`CMap2::to_vtk_file`].
fn build_unstructured_piece<T>(map: &CMap2<T>) -> UnstructuredGridPiece
where
    T: CoordsFloat + 'static,
{
    // common data
    let vertex_ids: Vec<VertexIdentifier> = map.fetch_vertices().identifiers;
    let mut id_map: BTreeMap<VertexIdentifier, usize> = BTreeMap::new();
    vertex_ids.iter().enumerate().for_each(|(id, vid)| {
        id_map.insert(*vid, id);
    });
    // ------ points data
    let vertices = vertex_ids
        .iter()
        .map(|vid| map.vertex(*vid).unwrap())
        .flat_map(|v| {
            let Coords2 { x, y } = v.into_inner();
            [x, y, T::zero()].into_iter()
        });
    // ------ cells data
    let mut n_cells = 0;
    // --- faces
    let face_ids = map.fetch_faces().identifiers;
    let face_data = face_ids.into_iter().map(|id| {
        let mut count: u32 = 0;
        // VecDeque will be useful later
        let orbit: Vec<u32> = Orbit2::new(map, OrbitPolicy::Face, id as DartIdentifier)
            .map(|dart_id| {
                count += 1;
                id_map[&map.vertex_id(dart_id)] as u32
            })
            .collect();
        (count, orbit)
    });

    // --- borders
    let edge_ids = map.fetch_edges().identifiers;
    // because we do not model boundaries, we can get edges
    // from filtering isolated darts making up edges
    let edge_data = edge_ids
        .into_iter()
        .filter(|id| map.beta::<2>(*id as DartIdentifier) == NULL_DART_ID)
        .map(|id| {
            let dart_id = id as DartIdentifier;
            let ndart_id = map.beta::<1>(dart_id);
            (
                id_map[&map.vertex_id(dart_id)] as u32,
                id_map[&map.vertex_id(ndart_id)] as u32,
            )
        });

    // --- corners
    // FIXME: ?
    // I'm not even sure corners can be detected without using additional attributes or metadata
    // let corner_data = vertex_ids.into_iter().filter(||)

    // ------ build VTK data
    let mut cell_vertices: Vec<u32> = Vec::new();
    let mut cell_types: Vec<CellType> = Vec::new();

    edge_data.for_each(|(v1, v2)| {
        cell_types.push(CellType::Line);
        cell_vertices.extend([2_u32, v1, v2]);
        n_cells += 1;
    });

    face_data.for_each(|(count, mut elements)| {
        cell_types.push(match count {
            0..=2 => return, // silent ignore
            3 => CellType::Triangle,
            4 => CellType::Quad,
            5.. => CellType::Polygon,
        });
        cell_vertices.push(count);
        cell_vertices.append(&mut elements);
        n_cells += 1;
    });

    UnstructuredGridPiece {
        points: if TypeId::of::<T>() == TypeId::of::<f32>() {
            IOBuffer::F32(vertices.map(|t| t.to_f32().unwrap()).collect())
        } else if TypeId::of::<T>() == TypeId::of::<f64>() {
            IOBuffer::F64(vertices.map(|t| t.to_f64().unwrap()).collect())
        } else {
            println!("W: unrecognized coordinate type -- cast to f64 might fail");
            IOBuffer::F64(vertices.map(|t| t.to_f64().unwrap()).collect())
        },
        cells: vtkio::model::Cells {
            cell_verts: VertexNumbers::Legacy {
                num_cells: n_cells,
                vertices: cell_vertices,
            },
            types: cell_types,
        },
        data: vtkio::model::Attributes::default(),
    }
}
