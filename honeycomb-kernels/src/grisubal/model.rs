//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::BBox2;
use honeycomb_core::{
    AttrSparseVec, AttributeBind, AttributeUpdate, CoordsFloat, DartIdentifier, OrbitPolicy,
    Vertex2, VertexIdentifier,
};
use num::Zero;
use vtkio::{
    model::{CellType, DataSet, VertexNumbers},
    IOBuffer, Vtk,
};

// ------ CONTENT

/// Post-processing clip operation.
#[derive(Default)]
pub enum Clip {
    /// Clip inner & outer cells, leaving only boundaries of the geometry.
    All,
    /// Clip inner cells.
    Inner,
    /// Clip outer cells.
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
///
/// For specification of the accepted VTK file format, see [`crate::grisubal`]'s documentation entry.
pub struct Geometry2<T: CoordsFloat> {
    /// Vertices of the geometry.
    pub vertices: Vec<Vertex2<T>>,
    /// Edges / segments making up the geometry.
    pub segments: Vec<(usize, usize)>,
    /// Points of interest, i.e. points to insert unconditionally in the future map / mesh.
    pub poi: Vec<usize>,
}

impl<T: CoordsFloat> Geometry2<T> {
    /// Return the bounding box of the geometry.
    pub fn bbox(&self) -> BBox2<T> {
        assert!(
            self.vertices.first().is_some(),
            "E: specified geometry does not contain any vertex"
        );
        let mut bbox = BBox2 {
            min_x: self.vertices[0].x(),
            max_x: self.vertices[0].x(),
            min_y: self.vertices[0].y(),
            max_y: self.vertices[0].y(),
        };

        self.vertices.iter().for_each(|v| {
            bbox.min_x = bbox.min_x.min(v.x());
            bbox.max_x = bbox.max_x.max(v.x()); // may not be optimal
            bbox.min_y = bbox.min_y.min(v.y()); // don't care
            bbox.max_y = bbox.max_y.max(v.y());
        });

        bbox
    }
}

macro_rules! build_vertices {
    ($v: ident) => {{
        assert!(
            ($v.len() % 3).is_zero(),
            "E: failed to build vertices list - the point list contains an incomplete tuple"
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

/// For specification of the accepted VTK file format, see [`crate::grisubal`]'s documentation entry.
impl<T: CoordsFloat> From<Vtk> for Geometry2<T> {
    fn from(value: Vtk) -> Self {
        // What we are reading / how we construct the geometry:
        // The input VTK file should describe boundaries (e.g. edges in 2D) & key vertices (e.g. sharp corners)
        // Those should be described by using simple
        match value.data {
            DataSet::ImageData { .. }
            | DataSet::StructuredGrid { .. }
            | DataSet::RectilinearGrid { .. }
            | DataSet::Field { .. } => {
                panic!("E: dataset not supported - only `UnstructuredGrid` is currently supported")
            }
            DataSet::PolyData { .. } => todo!("E: `PolyData` data set is not yet supported"),
            DataSet::UnstructuredGrid { pieces, .. } => {
                let mut vertices = Vec::new();
                let mut segments = Vec::new();
                let mut poi = Vec::new();
                let tmp = pieces.iter().map(|piece| {
                    // assume inline data
                    let tmp = piece
                        .load_piece_data(None)
                        .expect("E: failed to load piece data - is it not inlined?");

                    // build vertex list
                    // since we're expecting coordinates, we'll assume floating type
                    // we're also converting directly to our vertex type since we're building a 2-map
                    let vertices: Vec<Vertex2<T>> = match tmp.points {
                        IOBuffer::F64(v) => build_vertices!(v),
                        IOBuffer::F32(v) => build_vertices!(v),
                        _ => panic!("E: unsupported coordinate representation type - please use float or double"),
                    };
                    let mut poi: Vec<usize> = Vec::new();
                    let mut segments: Vec<(usize, usize)> = Vec::new();

                    let vtkio::model::Cells { cell_verts, types } = tmp.cells;
                    match cell_verts {
                        VertexNumbers::Legacy {
                            num_cells,
                            vertices: verts,
                        } => {
                            // check basic stuff
                            assert_eq!(num_cells as usize, types.len(),
                                "E: failed to build geometry - inconsistent number of cell between CELLS and CELL_TYPES"
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
                                    assert_eq!(vids.len(), 1,
                                        "E: failed to build geoemtry - `Vertex` cell has incorrect # of vertices (!=1)"
                                    );
                                    poi.push(vids[0]);
                                }
                                CellType::PolyVertex =>
                                    panic!("E: failed to build geometry - `PolyVertex` cell type is not supported, use `Vertex`s instead"),
                                CellType::Line => {
                                    assert_eq!(vids.len(), 2,
                                    "E: failed to build geometry - `Line` cell has incorrect # of vertices (!=2)"
                                );
                                    segments.push((vids[0],vids[1]));
                                }
                                CellType::PolyLine =>
                                    panic!("E: failed to build geometry - `PolyLine` cell type is not supported, use `Line`s instead"),
                                _ => {}, // silent ignore all other cells that do not make up boundaries
                            });
                        }
                        VertexNumbers::XML { .. } => {
                            panic!("E: XML Vtk files are not supported");
                        }
                    }
                    (vertices, segments, poi)
                });

                tmp.for_each(|(mut ver, mut seg, mut points)| {
                    vertices.append(&mut ver);
                    segments.append(&mut seg);
                    poi.append(&mut points);
                });

                Geometry2 {
                    vertices,
                    segments,
                    poi,
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum GeometryVertex {
    Regular(usize),
    PoI(usize),
    Intersec(VertexIdentifier),
}

#[derive(Clone, Copy, Debug)]
pub struct IsBoundary(bool);

impl AttributeUpdate for IsBoundary {
    fn merge(attr1: Self, attr2: Self) -> Self {
        // if we fuse two vertices and at least one is part of the boundary,
        // the resulting one should be part of the boundary to prevent a missing link in the chain
        IsBoundary(attr1.0 || attr2.0)
    }

    fn split(attr: Self) -> (Self, Self) {
        // if we split a vertex in two, both resulting vertices should hold the same property
        (attr, attr)
    }
}

impl AttributeBind for IsBoundary {
    fn binds_to<'a>() -> OrbitPolicy<'a> {
        OrbitPolicy::Vertex
    }

    type IdentifierType = VertexIdentifier;

    type StorageType = AttrSparseVec<Self>;
}
