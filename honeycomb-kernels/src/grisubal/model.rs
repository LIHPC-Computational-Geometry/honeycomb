//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::grisubal::GrisubalError;
use honeycomb_core::attributes::AttrSparseVec;
use honeycomb_core::cmap::CMapResult;
use honeycomb_core::prelude::{
    AttributeBind, AttributeUpdate, CoordsFloat, DartIdType, OrbitPolicy, Vertex2,
};
use vtkio::{
    model::{CellType, DataSet, VertexNumbers},
    IOBuffer, Vtk,
};

// ------ CONTENT

/// Structure used to index the overlapping grid's cases.
///
/// Cells `(X, Y)` take value in range `(0, 0)` to `(N, M)`,
/// from left to right (X), from bottom to top (Y).
#[derive(PartialEq, Clone, Copy)]
pub struct GridCellId(pub usize, pub usize);

impl GridCellId {
    /// Compute the [L1 / Manhattan distance](https://en.wikipedia.org/wiki/Taxicab_geometry) between
    /// two cells.
    pub fn l1_dist(lhs: &Self, rhs: &Self) -> usize {
        lhs.0.abs_diff(rhs.0) + lhs.1.abs_diff(rhs.1)
    }

    /// Compute the substraction between cell indices. This corresponds to an offset / movement over
    /// the grid **from `lhs` to `rhs`**.
    #[allow(clippy::cast_possible_wrap)]
    pub fn offset(lhs: &Self, rhs: &Self) -> (isize, isize) {
        (
            rhs.0 as isize - lhs.0 as isize,
            rhs.1 as isize - lhs.1 as isize,
        )
    }
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

macro_rules! build_vertices {
    ($v: ident) => {{
        if $v.len() % 3 != 0 {
            return Err(GrisubalError::BadVtkData(
                "vertex list contains an incomplete tuple",
            ));
        }
        $v.chunks_exact(3)
            .map(|slice| {
                // WE IGNORE Z values
                let &[x, y, _] = slice else { unreachable!() };
                Vertex2::from((T::from(x).unwrap(), T::from(y).unwrap()))
            })
            .collect()
    }};
}

/// For specification of the accepted VTK file format, see [`crate::grisubal`]'s documentation entry.
impl<T: CoordsFloat> TryFrom<Vtk> for Geometry2<T> {
    type Error = GrisubalError;

    #[allow(clippy::too_many_lines)]
    fn try_from(value: Vtk) -> Result<Self, Self::Error> {
        // What we are reading / how we construct the geometry:
        // The input VTK file should describe boundaries (e.g. edges in 2D) & key vertices (e.g. sharp corners)
        // Those should be described by using simple
        match value.data {
            DataSet::ImageData { .. }
            | DataSet::StructuredGrid { .. }
            | DataSet::RectilinearGrid { .. }
            | DataSet::Field { .. }
            | DataSet::PolyData { .. } => {
                Err(GrisubalError::UnsupportedVtkData("dataset not supported"))
            }
            DataSet::UnstructuredGrid { pieces, .. } => {
                let mut vertices = Vec::new();
                let mut segments = Vec::new();
                let mut poi = Vec::new();
                let tmp = pieces.iter().map(|piece| {
                    // assume inline data
                    let Ok(tmp) = piece.load_piece_data(None) else {
                        return Err(GrisubalError::UnsupportedVtkData("not inlined data piece"));
                    };

                    // build vertex list
                    // since we're expecting coordinates, we'll assume floating type
                    // we're also converting directly to our vertex type since we're building a 2-map
                    let vertices: Vec<Vertex2<T>> = match tmp.points {
                        IOBuffer::F64(v) => build_vertices!(v),
                        IOBuffer::F32(v) => build_vertices!(v),
                        _ => {
                            return Err(GrisubalError::UnsupportedVtkData(
                                "not float or double coordinate representation type",
                            ));
                        }
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
                            if num_cells as usize != types.len() {
                                return Err(GrisubalError::BadVtkData(
                                    "different # of cells in CELLS and CELL_TYPES",
                                ));
                            }

                            // build a collection of vertex lists corresponding of each cell
                            let mut cell_components: Vec<Vec<usize>> = Vec::new();
                            let mut take_next = 0;
                            for vertex_id in &verts {
                                if take_next == 0 {
                                    // making it usize since it's a counter
                                    take_next = *vertex_id as usize;
                                    cell_components.push(Vec::with_capacity(take_next));
                                } else {
                                    cell_components
                                        .last_mut()
                                        .expect("E: unreachable")
                                        .push(*vertex_id as usize);
                                    take_next -= 1;
                                }
                            }
                            assert_eq!(num_cells as usize, cell_components.len());

                            if let Some(err) = types.iter().zip(cell_components.iter()).find_map(
                                |(cell_type, vids)| match cell_type {
                                    CellType::Vertex => {
                                        if vids.len() != 1 {
                                            return Some(GrisubalError::BadVtkData(
                                                "`Vertex` with incorrect # of vertices (!=1)",
                                            ));
                                        }
                                        poi.push(vids[0]);
                                        None
                                    }
                                    CellType::PolyVertex => Some(
                                        GrisubalError::UnsupportedVtkData("`PolyVertex` cell type"),
                                    ),
                                    CellType::Line => {
                                        if vids.len() != 2 {
                                            return Some(GrisubalError::BadVtkData(
                                                "`Line` with incorrect # of vertices (!=2)",
                                            ));
                                        }
                                        segments.push((vids[0], vids[1]));
                                        None
                                    }
                                    CellType::PolyLine => {
                                        Some(GrisubalError::BadVtkData("`PolyLine` cell type"))
                                    }
                                    _ => None, // silent ignore all other cells that do not make up boundaries
                                },
                            ) {
                                return Err(err);
                            };
                        }
                        VertexNumbers::XML { .. } => {
                            return Err(GrisubalError::UnsupportedVtkData("XML format"));
                        }
                    }
                    Ok((vertices, segments, poi))
                });

                if let Some(e) = tmp.clone().find(Result::is_err) {
                    return Err(e.unwrap_err());
                }

                tmp.filter_map(Result::ok)
                    .for_each(|(mut ver, mut seg, mut points)| {
                        vertices.append(&mut ver);
                        segments.append(&mut seg);
                        poi.append(&mut points);
                    });

                Ok(Geometry2 {
                    vertices,
                    segments,
                    poi,
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GeometryVertex {
    /// Regular vertex. Inner `usize` indicates the vertex ID in-geometry.
    Regular(usize),
    /// Characteristic vertex, i.e. Point of Interest. Inner `usize` indicates the vertex ID in-geometry.
    PoI(usize),
    /// Intersection vertex. Inner `usize` indices the associated metadata ID in the dedicated collection.
    Intersec(usize),
    /// Intersection corner. This variant is dedicated to corner intersection and contain data that is directly
    /// used to instantiate [`MapEdge`] objects. The contained dart correspond to the intersected dart (end dart); the
    /// dart of the opposite quadrant (start dart of the next segment) can be retrieved by applying a combination of
    /// beta functions
    IntersecCorner(DartIdType),
}

#[derive(Debug)]
pub struct MapEdge<T: CoordsFloat> {
    pub start: DartIdType,
    pub intermediates: Vec<Vertex2<T>>,
    pub end: DartIdType,
}

/// Boundary-modeling enum.
///
/// This enum is used as an attribute (bound to single darts) to describe:
///
/// 1. if a dart is part of the captured geometry's boundary (`Left`/`Right` vs `None`)
/// 2. if it is, which side of the boundary it belongs to (`Left` vs `Right`)
///
/// The following image shows an oriented boundary (red), along with darts modeling its left side (purple),
/// right side (blue), and darts that do not model the boundary (black).
///
/// ![`DART_SIDES`](https://lihpc-computational-geometry.github.io/honeycomb/images/grisubal/left_right_darts.svg)
///
/// The attribute is set during the capture of the geometry so that it can be used at the (optional) clipping step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Boundary {
    /// Dart model the left side of the oriented boundary.
    Left,
    /// Dart model the right side of the oriented boundary.
    Right,
    /// Dart is not part of the boundary.
    None,
}

impl AttributeUpdate for Boundary {
    fn merge(attr1: Self, attr2: Self) -> Self {
        if attr1 == attr2 {
            attr1
        } else {
            Boundary::None
        }
    }

    fn split(_attr: Self) -> (Self, Self) {
        unreachable!()
    }

    fn merge_from_none() -> CMapResult<Self> {
        Ok(Boundary::None)
    }
}

impl AttributeBind for Boundary {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = DartIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}
