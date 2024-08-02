//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use std::collections::HashSet;

use crate::GrisubalError;

use honeycomb_core::{
    AttrSparseVec, AttributeBind, AttributeUpdate, CoordsFloat, DartIdentifier, EdgeIdentifier,
    OrbitPolicy, Vertex2, VertexIdentifier,
};
use num::Zero;
use vtkio::{
    model::{CellType, DataSet, VertexNumbers},
    IOBuffer, Vtk,
};

#[cfg(doc)]
use honeycomb_core::CMap2;

// ------ CONTENT

/// Post-processing clip operation.
///
/// Note that the part of the map that is clipped depends on the orientation of the original geometry provided as
/// input.
#[derive(Default)]
pub enum Clip {
    /// Clip all elements beside the captured boundary.
    All,
    /// Clip elements located on the left side of the oriented boundary.
    Left,
    /// Clip elements located on the right side of the oriented boundary.
    Right,
    /// Keep all elements. Default value.
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

/// Check for orientation issue **per boundary**.
///
/// This function check for the most obvious orientation issue; given a boundary, are all segments making it up
/// oriented consistently. If it is not the case, then there is at least one of:
///
/// - a vertex being the origin of two segment
/// - a vertex being the end-point of two segment
///
/// This does not cover consistent orientation across distinct boundaries (e.g. a geometry with a hole in it).
pub fn detect_orientation_issue<T: CoordsFloat>(
    geometry: &Geometry2<T>,
) -> Result<(), GrisubalError> {
    let mut origins = HashSet::new();
    let mut endpoints = HashSet::new();

    for (orig, endp) in &geometry.segments {
        if !origins.insert(orig) {
            return Err(GrisubalError::InconsistentOrientation(format!(
                "two segments have the same vertex as their respective origin: vertex #{}",
                *orig
            )));
        }
        if !endpoints.insert(endp) {
            return Err(GrisubalError::InconsistentOrientation(format!(
                "two segments have the same vertex as their respective end-point: vertex #{}",
                *endp
            )));
        }
    }

    Ok(())
}

pub fn compute_overlapping_grid<T: CoordsFloat>(
    geometry: &Geometry2<T>,
    [len_cell_x, len_cell_y]: [T; 2],
    allow_origin_offset: bool,
) -> ([usize; 2], Option<Vertex2<T>>) {
    // compute the minimum bounding box
    let (mut min_x, mut max_x, mut min_y, mut max_y): (T, T, T, T) = {
        let tmp = geometry
            .vertices
            .first()
            .expect("E: specified geometry does not contain any vertex");
        (tmp.x(), tmp.x(), tmp.y(), tmp.y())
    };

    geometry.vertices.iter().for_each(|v| {
        min_x = min_x.min(v.x());
        max_x = max_x.max(v.x()); // may not be optimal
        min_y = min_y.min(v.y()); // don't care
        max_y = max_y.max(v.y());
    });

    // compute characteristics of the overlapping Cartesian grid
    if allow_origin_offset {
        todo!()
    } else {
        assert!(
            min_x > T::zero(),
            "E: the geometry should be entirely defined in positive Xs/Ys"
        );
        assert!(
            min_y > T::zero(),
            "E: the geometry should be entirely defined in positive Xs/Ys"
        );
        assert!(max_x > min_x);
        assert!(max_y > min_y);
        let n_cells_x = (max_x / len_cell_x).ceil().to_usize().unwrap() + 1;
        let n_cells_y = (max_y / len_cell_y).ceil().to_usize().unwrap() + 1;
        ([n_cells_x, n_cells_y], None)
    }
}

/// Remove from their geometry points of interest that intersect with a grid of specified dimension.
///
/// This function works under the assumption that the grid is Cartesian & has its origin on `(0.0, 0.0)`.
pub fn remove_redundant_poi<T: CoordsFloat>(geometry: &mut Geometry2<T>, [cx, cy]: [T; 2]) {
    // PoI that land on the grid create a number of issues; removing them is ok since we're intersecting the grid
    // at their coordinates, so the shape will be captured via intersection anyway
    geometry.poi.retain(|idx| {
        let v = geometry.vertices[*idx];
        // origin is assumed to be (0.0, 0.0)
        let on_x_axis = (v.x() % cx).is_zero();
        let on_y_axis = (v.y() % cy).is_zero();
        !(on_x_axis | on_y_axis)
    });
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
    IntersecCorner(DartIdentifier),
}

pub struct MapEdge<T: CoordsFloat> {
    pub start: DartIdentifier,
    pub intermediates: Vec<Vertex2<T>>,
    pub end: DartIdentifier,
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

    fn split(attr: Self) -> (Self, Self) {
        unreachable!()
    }

    fn merge_undefined(attr: Option<Self>) -> Self {
        attr.unwrap_or(Boundary::None)
    }
}

impl AttributeBind for Boundary {
    fn binds_to<'a>() -> OrbitPolicy<'a> {
        OrbitPolicy::Custom(&[])
    }

    type IdentifierType = DartIdentifier;

    type StorageType = AttrSparseVec<Self>;
}
