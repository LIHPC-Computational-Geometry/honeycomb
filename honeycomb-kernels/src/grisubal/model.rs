//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use std::collections::HashSet;

use honeycomb_core::attributes::AttrSparseVec;
use honeycomb_core::prelude::{
    AttributeBind, AttributeUpdate, CoordsFloat, DartIdentifier, OrbitPolicy, Vertex2,
};

use vtkio::{
    model::{CellType, DataSet, VertexNumbers},
    IOBuffer, Vtk,
};

use crate::grisubal::grid::GridCellId;
use crate::grisubal::GrisubalError;
#[cfg(doc)]
use honeycomb_core::prelude::CMap2;

// ------ CONTENT

/// Post-processing clip operation.
///
/// Note that the part of the map that is clipped depends on the orientation of the original geometry provided as
/// input.
#[derive(Default)]
pub enum Clip {
    /// Clip elements located on the left side of the oriented boundary.
    Left,
    /// Clip elements located on the right side of the oriented boundary.
    Right,
    /// Keep all elements. Default value.
    #[default]
    None,
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
        if !origins.insert(orig) || !endpoints.insert(endp) {
            return Err(GrisubalError::InconsistentOrientation(
                "in-boundary inconsistency",
            ));
        }
    }

    Ok(())
}

#[allow(clippy::cast_precision_loss)]
pub fn compute_overlapping_grid<T: CoordsFloat>(
    geometry: &Geometry2<T>,
    [len_cell_x, len_cell_y]: [T; 2],
) -> Result<([usize; 2], Vertex2<T>), GrisubalError> {
    // compute the minimum bounding box
    let (mut min_x, mut max_x, mut min_y, mut max_y): (T, T, T, T) = {
        let Some(tmp) = geometry.vertices.first() else {
            return Err(GrisubalError::InvalidShape("no vertex in shape"));
        };

        (tmp.x(), tmp.x(), tmp.y(), tmp.y())
    };

    geometry.vertices.iter().for_each(|v| {
        min_x = min_x.min(v.x());
        max_x = max_x.max(v.x()); // may not be optimal
        min_y = min_y.min(v.y()); // don't care
        max_y = max_y.max(v.y());
    });

    if max_x <= min_x {
        return Err(GrisubalError::InvalidShape(
            "bounding values along X axis are equal",
        ));
    }
    if max_y <= min_y {
        return Err(GrisubalError::InvalidShape(
            "bounding values along Y axis are equal",
        ));
    }

    // compute characteristics of the overlapping Cartesian grid

    // create a ~one-and-a-half cell buffer to contain the geometry
    // this, along with the `+1` below, guarantees that
    // dart at the boundary of the grid are not intersected by the geometry
    let mut og_x = min_x - len_cell_x * T::from(1.5).unwrap();
    let mut og_y = min_y - len_cell_y * T::from(1.5).unwrap();
    // we check for some extremely annoying cases here
    // if some are detected, the origin is incrementally shifted
    let (mut on_corner, mut reflect) =
        detect_overlaps(geometry, [len_cell_x, len_cell_y], Vertex2(og_x, og_y));
    let mut i = 1;

    while on_corner | reflect {
        eprintln!(
            "W: land on corner: {on_corner} - reflect on an axis: {reflect}, shifting origin"
        );
        og_x += len_cell_x * T::from(1. / (2_i32.pow(i + 1) as f32)).unwrap();
        og_y += len_cell_y * T::from(1. / (2_i32.pow(i + 1) as f32)).unwrap();
        (on_corner, reflect) =
            detect_overlaps(geometry, [len_cell_x, len_cell_y], Vertex2(og_x, og_y));
        i += 1;
    }

    let n_cells_x = ((max_x - og_x) / len_cell_x).ceil().to_usize().unwrap() + 1;
    let n_cells_y = ((max_y - og_y) / len_cell_y).ceil().to_usize().unwrap() + 1;

    Ok(([n_cells_x, n_cells_y], Vertex2(og_x, og_y)))
}

/// Remove from their geometry points of interest that intersect with a grid of specified dimension.
///
/// This function works under the assumption that the grid is Cartesian & has its origin on `(0.0, 0.0)`.
pub fn remove_redundant_poi<T: CoordsFloat>(
    geometry: &mut Geometry2<T>,
    [cx, cy]: [T; 2],
    origin: Vertex2<T>,
) {
    // PoI that land on the grid create a number of issues; removing them is ok since we're intersecting the grid
    // at their coordinates, so the shape will be captured via intersection anyway
    geometry.poi.retain(|idx| {
        let v = geometry.vertices[*idx];
        // origin is assumed to be (0.0, 0.0)
        let on_x_axis = ((v.x() - origin.x()) % cx).is_zero();
        let on_y_axis = ((v.y() - origin.y()) % cy).is_zero();
        !(on_x_axis | on_y_axis)
    });
}

pub fn detect_overlaps<T: CoordsFloat>(
    geometry: &Geometry2<T>,
    [cx, cy]: [T; 2],
    origin: Vertex2<T>,
) -> (bool, bool) {
    let on_corner = geometry
        .vertices
        .iter()
        .map(|v| {
            let on_x_axis = ((v.x() - origin.x()) % cx).is_zero();
            let on_y_axis = ((v.y() - origin.y()) % cy).is_zero();
            on_x_axis && on_y_axis
        })
        .any(|a| a);

    let bad_reflection = geometry
        .vertices
        .iter()
        .enumerate()
        .filter_map(|(id, v)| {
            let on_x_axis = ((v.x() - origin.x()) % cx).is_zero();
            let on_y_axis = ((v.y() - origin.y()) % cy).is_zero();
            if on_x_axis | on_y_axis {
                return Some(id);
            }
            None
        })
        // skip vertices that do not belong to the boundary
        .filter(|id| {
            geometry
                .segments
                .iter()
                .any(|(v1, v2)| (id == v1) || (id == v2))
        })
        .map(|id| {
            // if a vertex appear in the boundary, there should be both a segment landing and a
            // segment starting on the vertex; hence `.expect()`
            let vid_in = geometry
                .segments
                .iter()
                .find_map(|(vin, ref_id)| {
                    if id == *ref_id {
                        return Some(*vin);
                    }
                    None
                })
                .expect("E: found a vertex with no incident segment - is the geometry open?");
            // same
            let vid_out = geometry
                .segments
                .iter()
                .find_map(|(ref_id, vout)| {
                    if id == *ref_id {
                        return Some(*vout);
                    }
                    None
                })
                .expect("E: found a vertex with no incident segment - is the geometry open?");
            let v_in = geometry.vertices[vid_in];
            let v_out = geometry.vertices[vid_out];
            let Vertex2(ox, oy) = origin;
            let (c_in, c_out) = (
                GridCellId(
                    ((v_in.x() - ox) / cx).floor().to_usize().unwrap(),
                    ((v_in.y() - oy) / cy).floor().to_usize().unwrap(),
                ),
                GridCellId(
                    ((v_out.x() - ox) / cx).floor().to_usize().unwrap(),
                    ((v_out.y() - oy) / cy).floor().to_usize().unwrap(),
                ),
            );
            // if v_in and v_out belong to the same grid cell, there was a "reflection" on one
            // of the grid's axis
            c_in == c_out
        })
        .any(|a| a);

    (on_corner, bad_reflection)
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

#[derive(Debug)]
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

    fn split(_attr: Self) -> (Self, Self) {
        unreachable!()
    }

    fn merge_from_none() -> Option<Self> {
        Some(Boundary::None)
    }
}

impl AttributeBind for Boundary {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = DartIdentifier;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}
