use crate::cmap::{EdgeIdType, FaceIdType};
use crate::geometry::CoordsFloat;
use crate::prelude::{CMap2, DartIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID};

use std::{any::TypeId, collections::BTreeMap};

use vtkio::{
    model::{
        ByteOrder, CellType, DataSet, Piece, UnstructuredGridPiece, Version, VertexNumbers, Vtk,
    },
    IOBuffer,
};

// --- VTK

/// **Serialization methods**
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
            version: Version::Legacy { major: 2, minor: 0 },
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
            .expect("E: could not write data to writer");
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
            version: Version::Legacy { major: 2, minor: 0 },
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
            .expect("E: could not write data to writer");
    }
}

/// Internal building routine for VTK serialization.
fn build_unstructured_piece<T>(map: &CMap2<T>) -> UnstructuredGridPiece
where
    T: CoordsFloat + 'static,
{
    // common data
    let vertex_ids: Vec<VertexIdType> = map.iter_vertices().collect();
    let mut id_map: BTreeMap<VertexIdType, usize> = BTreeMap::new();
    vertex_ids.iter().enumerate().for_each(|(id, vid)| {
        id_map.insert(*vid, id);
    });
    // ------ points data
    let vertices = vertex_ids
        .iter()
        .map(|vid| {
            map.force_read_vertex(*vid)
                .expect("E: found a topological vertex with no associated coordinates")
        })
        .flat_map(|v| [v.x(), v.y(), T::zero()].into_iter());
    // ------ cells data
    let mut n_cells = 0;
    // --- faces
    let face_ids: Vec<FaceIdType> = map.iter_faces().collect();
    let face_data = face_ids.into_iter().map(|id| {
        let mut count: u32 = 0;
        // VecDeque will be useful later
        let orbit: Vec<u32> = Orbit2::new(map, OrbitPolicy::Custom(&[1]), id as DartIdType)
            .map(|dart_id| {
                count += 1;
                id_map[&map.vertex_id(dart_id)] as u32
            })
            .collect();
        (count, orbit)
    });

    // --- borders
    let edge_ids: Vec<EdgeIdType> = map.iter_edges().collect();
    // because we do not model boundaries, we can get edges
    // from filtering isolated darts making up edges
    let edge_data = edge_ids
        .into_iter()
        .filter(|id| map.beta::<2>(*id as DartIdType) == NULL_DART_ID)
        .map(|id| {
            let dart_id = id as DartIdType;
            let ndart_id = map.beta::<1>(dart_id);
            (
                id_map[&map.vertex_id(dart_id)] as u32,
                id_map[&map.vertex_id(ndart_id)] as u32,
            )
        });

    // --- corners

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
            IOBuffer::F32(
                vertices
                    .map(|t| t.to_f32().expect("E: unreachable"))
                    .collect(),
            )
        } else if TypeId::of::<T>() == TypeId::of::<f64>() {
            IOBuffer::F64(
                vertices
                    .map(|t| t.to_f64().expect("E: unreachable"))
                    .collect(),
            )
        } else {
            println!("W: unrecognized coordinate type -- cast to f64 might fail");
            IOBuffer::F64(
                vertices
                    .map(|t| t.to_f64().expect("E: unreachable"))
                    .collect(),
            )
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
