//! Input/Output features implementation
//!
//! The support for I/O is currently very restricted since this is not the focus of this project.
//! Maps can be built from and serialized to VTK legacy files (both binary and ASCII). The
//! `DATASET` of the VTK file should be `UNSTRUCTURED_GRID`, and only a given set of `CELL_TYPES`
//! are supported, because of orientation and dimension restriction.

// ------ IMPORTS

use crate::geometry::CoordsFloat;
use crate::prelude::{CMap2, DartIdentifier, Orbit2, OrbitPolicy, VertexIdentifier, NULL_DART_ID};

use std::{any::TypeId, collections::BTreeMap};

use vtkio::{
    model::{
        ByteOrder, CellType, DataSet, Piece, UnstructuredGridPiece, Version, VertexNumbers, Vtk,
    },
    IOBuffer,
};

// ------ CONTENT

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
            .expect("Could not write data to writer");
    }
}

// --- internals

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
        .flat_map(|v| [v.x(), v.y(), T::zero()].into_iter());
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
