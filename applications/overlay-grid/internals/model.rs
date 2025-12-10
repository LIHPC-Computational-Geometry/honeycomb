use std::collections::HashSet;

use honeycomb::core::{
    attributes::{AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate},
    cmap::{CMap2, CMapBuilder, DartIdType, FaceIdType, OrbitPolicy},
    geometry::{CoordsFloat, Vertex2},
};
use thiserror::Error;
use vtkio::{
    IOBuffer, Vtk,
    model::{CellType, DataSet, VertexNumbers},
};

#[derive(Error, Debug)]
pub enum VtkError {
    /// An orientation issue has been detected in the input geometry.
    #[error("boundary isn't consistently oriented - {0}")]
    InconsistentOrientation(&'static str),
    /// The specified geometry does not match one (or more) requirements of the algorithm.
    #[error("input shape isn't conform to requirements - {0}")]
    InvalidShape(&'static str),
    /// The VTK file used to try to build a `Geometry2` object contains invalid data
    /// (per VTK's specification).
    #[error("invalid/corrupted data in the vtk file - {0}")]
    BadVtkData(&'static str),
    /// The VTK file used to try to build a `Geometry2` object contains valid but unsupported data.
    #[error("unsupported data in the vtk file - {0}")]
    UnsupportedVtkData(&'static str),
}

// =================================================================================================
// Geometry Data Structures
// =================================================================================================

pub struct Geometry2<T: CoordsFloat> {
    /// Vertices of the geometry.
    pub vertices: Vec<Vertex2<T>>,
    /// Edges / segments making up the geometry.
    pub segments: Vec<(usize, usize)>,
}

macro_rules! build_vertices {
    ($v: ident) => {{
        if $v.len() % 3 != 0 {
            return Err(VtkError::BadVtkData(
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

/// For specification of the accepted VTK file format, see [`crate`]'s documentation entry.
impl<T: CoordsFloat> TryFrom<Vtk> for Geometry2<T> {
    type Error = VtkError;

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
                Err(VtkError::UnsupportedVtkData("dataset not supported"))
            }
            DataSet::UnstructuredGrid { pieces, .. } => {
                let mut vertices = Vec::new();
                let mut segments = Vec::new();
                let tmp = pieces.iter().map(|piece| {
                    // assume inline data
                    let Ok(tmp) = piece.load_piece_data(None) else {
                        return Err(VtkError::UnsupportedVtkData("not inlined data piece"));
                    };

                    // build vertex list
                    // since we're expecting coordinates, we'll assume floating type
                    // we're also converting directly to our vertex type since we're building a 2-map
                    let vertices: Vec<Vertex2<T>> = match tmp.points {
                        IOBuffer::F64(v) => build_vertices!(v),
                        IOBuffer::F32(v) => build_vertices!(v),
                        _ => {
                            return Err(VtkError::UnsupportedVtkData(
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
                                return Err(VtkError::BadVtkData(
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
                                            return Some(VtkError::BadVtkData(
                                                "`Vertex` with incorrect # of vertices (!=1)",
                                            ));
                                        }
                                        poi.push(vids[0]);
                                        None
                                    }
                                    CellType::PolyVertex => {
                                        Some(VtkError::UnsupportedVtkData("`PolyVertex` cell type"))
                                    }
                                    CellType::Line => {
                                        if vids.len() != 2 {
                                            return Some(VtkError::BadVtkData(
                                                "`Line` with incorrect # of vertices (!=2)",
                                            ));
                                        }
                                        segments.push((vids[0], vids[1]));
                                        None
                                    }
                                    CellType::PolyLine => {
                                        Some(VtkError::BadVtkData("`PolyLine` cell type"))
                                    }
                                    _ => None, // silent ignore all other cells that do not make up boundaries
                                },
                            ) {
                                return Err(err);
                            }
                        }
                        VertexNumbers::XML { .. } => {
                            return Err(VtkError::UnsupportedVtkData("XML format"));
                        }
                    }
                    Ok((vertices, segments))
                });

                if let Some(e) = tmp.clone().find(Result::is_err) {
                    return Err(e.unwrap_err());
                }

                tmp.filter_map(Result::ok).for_each(|(mut ver, mut seg)| {
                    vertices.append(&mut ver);
                    segments.append(&mut seg);
                });

                Ok(Geometry2 { vertices, segments })
            }
        }
    }
}

// =================================================================================================
// Custom Attributes
// =================================================================================================

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct IsIrregular(pub bool);

impl AttributeUpdate for IsIrregular {
    fn merge(attr1: Self, _attr2: Self) -> Result<Self, AttributeError> {
        Ok(attr1)
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}

impl AttributeBind for IsIrregular {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = DartIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Edge;
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct GeoVertices(pub (u32, u32));

impl AttributeUpdate for GeoVertices {
    fn merge(attr1: Self, _attr2: Self) -> Result<Self, AttributeError> {
        Ok(attr1)
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}

impl AttributeBind for GeoVertices {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = FaceIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Face;
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct SiblingDartId(pub u32);

impl AttributeUpdate for SiblingDartId {
    fn merge(attr1: Self, _attr2: Self) -> Result<Self, AttributeError> {
        Ok(attr1)
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}

impl AttributeBind for SiblingDartId {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = FaceIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Face;
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct RefinementLevel(pub u16);

impl AttributeUpdate for RefinementLevel {
    fn merge(attr1: Self, _attr2: Self) -> Result<Self, AttributeError> {
        Ok(attr1)
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}

impl AttributeBind for RefinementLevel {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = FaceIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Face;
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct B2Mapping(pub u32);

impl AttributeUpdate for B2Mapping {
    fn merge(attr1: Self, _attr2: Self) -> Result<Self, AttributeError> {
        Ok(attr1)
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}

impl AttributeBind for B2Mapping {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = DartIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Edge;
}

// =================================================================================================
// Preprocessing Functions
// =================================================================================================

/// Check for orientation issue **per boundary**.
///
/// This function check for the most obvious orientation issue; given a boundary, are all segments making it up
/// oriented consistently. If it is not the case, then there is at least one of:
///
/// - a vertex being the origin of two segment
/// - a vertex being the end-point of two segment
///
/// This does not cover consistent orientation across distinct boundaries (e.g. a geometry with a hole in it).
pub fn detect_orientation_issue<T: CoordsFloat>(geometry: &Geometry2<T>) -> Result<(), VtkError> {
    let mut origins = HashSet::new();
    let mut endpoints = HashSet::new();

    for (orig, endp) in &geometry.segments {
        if !origins.insert(orig) || !endpoints.insert(endp) {
            return Err(VtkError::InconsistentOrientation(
                "in-boundary inconsistency",
            ));
        }
    }

    Ok(())
}

pub fn compute_overlapping_grid_size<T: CoordsFloat>(
    geometry: &Geometry2<T>,
) -> Result<[T; 4], VtkError> {
    // compute the minimum bounding box
    let (mut min_x, mut max_x, mut min_y, mut max_y): (T, T, T, T) = {
        let Some(tmp) = geometry.vertices.first() else {
            return Err(VtkError::InvalidShape("no vertex in shape"));
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
        return Err(VtkError::InvalidShape(
            "bounding values along X axis are equal",
        ));
    }
    if max_y <= min_y {
        return Err(VtkError::InvalidShape(
            "bounding values along Y axis are equal",
        ));
    }

    Ok([max_x, max_y, min_x, min_y])
}

// =================================================================================================
// Grid Initialization
// =================================================================================================

pub fn manual_grid<T: CoordsFloat>(nb_verts: usize) -> (CMap2<T>, Vec<Vertex2<T>>) {
    // Generate random geo vertices within the -10 to 10 box
    use rand::Rng;
    let mut rng = rand::rng();
    let geo_verts: Vec<Vertex2<T>> = (0..nb_verts)
        .map(|_| {
            let x = T::from(rng.random_range(-10.0..=10.0)).unwrap();
            let y = T::from(rng.random_range(-10.0..=10.0)).unwrap();
            Vertex2(x, y)
        })
        .collect();

    // --- BUILD THE CMap2
    let map: CMap2<T> = CMapBuilder::<2>::from_n_darts(4)
        .add_attribute::<IsIrregular>()
        .add_attribute::<GeoVertices>()
        .add_attribute::<SiblingDartId>()
        .add_attribute::<RefinementLevel>()
        .add_attribute::<B2Mapping>()
        .build()
        .unwrap();

    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_link::<1>(4, 1).unwrap();

    map.force_write_vertex(1, Vertex2(T::from(-10.0).unwrap(), T::from(-10.0).unwrap()));
    map.force_write_vertex(2, Vertex2(T::from(10.0).unwrap(), T::from(-10.0).unwrap()));
    map.force_write_vertex(3, Vertex2(T::from(10.0).unwrap(), T::from(10.0).unwrap()));
    map.force_write_vertex(4, Vertex2(T::from(-10.0).unwrap(), T::from(10.0).unwrap()));

    map.force_write_attribute::<GeoVertices>(1, GeoVertices((0u32, geo_verts.len() as u32)));
    map.force_write_attribute::<RefinementLevel>(1, RefinementLevel(0));

    (map, geo_verts)
}

pub fn vtk_grid<T: CoordsFloat>(file_path: &str) -> Result<(CMap2<T>, Vec<Vertex2<T>>), VtkError> {
    // --- IMPORT VTK INPUT
    let geometry_vtk = match Vtk::import(file_path) {
        Ok(vtk) => vtk,
        Err(e) => panic!("E: could not open specified vtk file - {e}"),
    };
    //----/

    // --- BUILD OUR MODEL FROM THE VTK IMPORT
    let geometry = Geometry2::<T>::try_from(geometry_vtk)?;
    //----/

    // --- FIRST DETECTION OF ORIENTATION ISSUES
    detect_orientation_issue(&geometry)?;
    //----/

    // --- Overlapping grid size computation
    let [max_x, max_y, min_x, min_y] = compute_overlapping_grid_size(&geometry)?;
    let _quadtree_bounds = [min_x, min_y, max_x, max_y];
    let width = max_x - min_x;
    let height = max_y - min_y;

    let padding_factor = T::from(0.1).unwrap();
    let x_padding = width * padding_factor;
    let y_padding = height * padding_factor;

    let grid_min_x = min_x - x_padding;
    let grid_max_x = max_x + x_padding;
    let grid_min_y = min_y - y_padding;
    let grid_max_y = max_y + y_padding;

    // Extract geo vertices from the geometry
    let geo_verts = geometry.vertices;

    // --- BUILD THE CMap2
    let map: CMap2<T> = CMapBuilder::<2>::from_n_darts(4)
        .add_attribute::<IsIrregular>()
        .add_attribute::<GeoVertices>()
        .add_attribute::<SiblingDartId>()
        .add_attribute::<RefinementLevel>()
        .add_attribute::<B2Mapping>()
        .build()
        .unwrap();

    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_link::<1>(4, 1).unwrap();

    // Set vertices based on geometry bounds with padding
    map.force_write_vertex(1, Vertex2(grid_min_x, grid_min_y)); // Bottom-left
    map.force_write_vertex(2, Vertex2(grid_max_x, grid_min_y)); // Bottom-right
    map.force_write_vertex(3, Vertex2(grid_max_x, grid_max_y)); // Top-right
    map.force_write_vertex(4, Vertex2(grid_min_x, grid_max_y)); // Top-left

    map.force_write_attribute::<GeoVertices>(1, GeoVertices((0u32, geo_verts.len() as u32)));
    map.force_write_attribute::<RefinementLevel>(1, RefinementLevel(0));

    Ok((map, geo_verts))
}
