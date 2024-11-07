//! Input/Output features implementation
//!
//! The support for I/O is currently very restricted since this is not the focus of this project.
//! Maps can be built from and serialized to VTK legacy files (both binary and ASCII). The
//! `DATASET` of the VTK file should be `UNSTRUCTURED_GRID`, and only a given set of `CELL_TYPES`
//! are supported, because of orientation and dimension restriction.

// ------ IMPORTS

use std::collections::BTreeMap;

use crate::{
    attributes::AttrStorageManager,
    cmap::{BuilderError, CMap2, CMapBuilder, DartId, DartIdType},
    geometry::{CoordsFloat, Vertex2},
};
use num_traits::Zero;
use vtkio::model::{CellType, DataSet, VertexNumbers};
use vtkio::{IOBuffer, Vtk};

// ------ CONTENT

impl<T: CoordsFloat> CMapBuilder<T> {
    /// Import and set the VTK file that will be used when building the map.
    ///
    /// # Panics
    ///
    /// This function may panic if the file cannot be loaded.
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn vtk_file(mut self, file_path: impl AsRef<std::path::Path> + std::fmt::Debug) -> Self {
        let vtk_file =
            Vtk::import(file_path).unwrap_or_else(|e| panic!("E: failed to load file: {e:?}"));
        self.vtk_file = Some(vtk_file);
        self
    }
}

/// Create a [`CMapBuilder`] from an imported VTK file.
///
/// This implementation is roughly equivalent to the following:
///
/// ```rust,no_run
/// # use honeycomb_core::prelude::CMapBuilder;
/// // `CMapBuilder::from_vtk_file("some/path/to/file.vtk")`, or:
/// let builder = CMapBuilder::<f64>::default().vtk_file("some/path/to/file.vtk");
/// ```
///
/// # Panics
///
/// This function may panic if the file cannot be loaded.
impl<T: CoordsFloat, P: AsRef<std::path::Path> + std::fmt::Debug> From<P> for CMapBuilder<T> {
    fn from(value: P) -> Self {
        let vtk_file =
            Vtk::import(value).unwrap_or_else(|e| panic!("E: failed to load file: {e:?}"));
        CMapBuilder {
            vtk_file: Some(vtk_file),
            ..Default::default()
        }
    }
}

macro_rules! if_predicate_return_err {
    ($pr: expr, $er: expr) => {
        if $pr {
            return Err($er);
        }
    };
}

macro_rules! build_vertices {
    ($v: ident) => {{
        if_predicate_return_err!(
            !($v.len() % 3).is_zero(),
            BuilderError::BadVtkData("vertex list contains an incomplete tuple")
        );
        $v.chunks_exact(3)
            .map(|slice| {
                // WE IGNORE Z values
                let &[x, y, _] = slice else { unreachable!() };
                Vertex2(T::from(x).unwrap(), T::from(y).unwrap())
            })
            .collect()
    }};
}

#[allow(clippy::too_many_lines)]
/// Internal building routine for [`CMap2::from_vtk_file`].
///
/// # Result / Errors
///
/// This implementation support only a very specific subset of VTK files. This result in many
/// possibilities for failure. This function may return:
///
/// - `Ok(CMap2)` -- The file was successfully parsed and its content made into a 2-map.
/// - `Err(BuilderError)` -- The function failed for one of the following reasons (sorted
///   by [`BuilderError`] variants):
///     - `UnsupportedVtkData`: The file contains unsupported data, i.e.:
///         - file format isn't Legacy,
///         - data set is something other than `UnstructuredGrid`,
///         - coordinate representation type isn't `float` or `double`
///         - mesh contains unsupported cell types (`PolyVertex`, `PolyLine`, `TriangleStrip`,
///           `Pixel` or anything 3D)
///     - `InvalidVtkFile`: The file contains inconsistencies, i.e.:
///         - the number of coordinates cannot be divided by `3`, meaning a tuple is incomplete
///         - the number of `Cells` and `CellTypes` isn't equal
///         - a given cell has an inconsistent number of vertices with its specified cell type
pub fn build_2d_from_vtk<T: CoordsFloat>(
    value: Vtk,
    mut _manager: AttrStorageManager, // FIXME: find a cleaner solution to populate the manager
) -> Result<CMap2<T>, BuilderError> {
    let mut cmap: CMap2<T> = CMap2::new(0);
    let mut sew_buffer: BTreeMap<(usize, usize), DartId> = BTreeMap::new();
    match value.data {
        DataSet::ImageData { .. }
        | DataSet::StructuredGrid { .. }
        | DataSet::RectilinearGrid { .. }
        | DataSet::PolyData { .. }
        | DataSet::Field { .. } => {
            return Err(BuilderError::UnsupportedVtkData("dataset not supported"))
        }
        DataSet::UnstructuredGrid { pieces, .. } => {
            let mut tmp = pieces.iter().map(|piece| {
                // assume inline data
                let Ok(tmp) = piece.load_piece_data(None) else {
                    return Err(BuilderError::UnsupportedVtkData("not inlined data piece"));
                };

                // build vertex list
                // since we're expecting coordinates, we'll assume floating type
                // we're also converting directly to our vertex type since we're building a 2-map
                let vertices: Vec<Vertex2<T>> = match tmp.points {
                    IOBuffer::F64(v) => build_vertices!(v),
                    IOBuffer::F32(v) => build_vertices!(v),
                    _ => {
                        return Err(BuilderError::UnsupportedVtkData(
                            "unsupported coordinate type",
                        ))
                    }
                };

                let vtkio::model::Cells { cell_verts, types } = tmp.cells;
                match cell_verts {
                    VertexNumbers::Legacy {
                        num_cells,
                        vertices: verts,
                    } => {
                        // check basic stuff
                        if_predicate_return_err!(
                            num_cells as usize != types.len(),
                            BuilderError::BadVtkData("different # of cell in CELLS and CELL_TYPES")
                        );

                        // build a collection of vertex lists corresponding of each cell
                        let mut cell_components: Vec<Vec<usize>> = Vec::new();
                        let mut take_next = 0;
                        for vertex_id in &verts {
                            if take_next.is_zero() {
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

                        let mut errs =
                            types
                                .iter()
                                .zip(cell_components.iter())
                                .map(|(cell_type, vids)| match cell_type {
                                    CellType::Vertex => {
                                        if_predicate_return_err!(
                                            vids.len() != 1,
                                            BuilderError::BadVtkData(
                                                "`Vertex` with incorrect # of vertices (!=1)"
                                            )
                                        );
                                        // silent ignore
                                        Ok(())
                                    }
                                    CellType::PolyVertex => Err(BuilderError::UnsupportedVtkData(
                                        "`PolyVertex` cell type",
                                    )),
                                    CellType::Line => {
                                        if_predicate_return_err!(
                                            vids.len() != 2,
                                            BuilderError::BadVtkData(
                                                "`Line` with incorrect # of vertices (!=2)"
                                            )
                                        );
                                        // silent ignore
                                        Ok(())
                                    }
                                    CellType::PolyLine => Err(BuilderError::UnsupportedVtkData(
                                        "`PolyLine` cell type",
                                    )),
                                    CellType::Triangle => {
                                        // check validity
                                        if_predicate_return_err!(
                                            vids.len() != 3,
                                            BuilderError::BadVtkData(
                                                "`Triangle` with incorrect # of vertices (!=3)"
                                            )
                                        );
                                        // build the triangle
                                        let d0 = cmap.add_free_darts(3);
                                        let (d1, d2) = (DartId(d0.0 + 1), DartId(d0.0 + 2));
                                        cmap.insert_vertex(d0.into(), vertices[vids[0]]);
                                        cmap.insert_vertex(d1.into(), vertices[vids[1]]);
                                        cmap.insert_vertex(d2.into(), vertices[vids[2]]);

                                        cmap.one_link(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                                        cmap.one_link(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                                        cmap.one_link(d2, d0); // edge d2 links vertices vids[2] & vids[0]

                                        // record a trace of the built cell for future 2-sew
                                        sew_buffer.insert((vids[0], vids[1]), d0);
                                        sew_buffer.insert((vids[1], vids[2]), d1);
                                        sew_buffer.insert((vids[2], vids[0]), d2);
                                        Ok(())
                                    }
                                    CellType::TriangleStrip => {
                                        Err(BuilderError::UnsupportedVtkData(
                                            "`TriangleStrip` cell type",
                                        ))
                                    }
                                    CellType::Polygon => {
                                        let n_vertices = vids.len();
                                        let d0 = cmap.add_free_darts(n_vertices);
                                        (0..n_vertices).for_each(|i| {
                                            let di = DartId(d0.0 + i as DartIdType);
                                            let dip1 = if i == n_vertices - 1 {
                                                d0
                                            } else {
                                                DartId(di.0 + 1)
                                            };
                                            cmap.insert_vertex(di.into(), vertices[vids[i]]);
                                            cmap.one_link(di, dip1);
                                            sew_buffer
                                                .insert((vids[i], vids[(i + 1) % n_vertices]), di);
                                        });
                                        Ok(())
                                    }
                                    CellType::Pixel => {
                                        Err(BuilderError::UnsupportedVtkData("`Pixel` cell type"))
                                    }
                                    CellType::Quad => {
                                        if_predicate_return_err!(
                                            vids.len() != 4,
                                            BuilderError::BadVtkData(
                                                "`Quad` with incorrect # of vertices (!=4)"
                                            )
                                        );
                                        // build the quad
                                        let d0 = cmap.add_free_darts(4);
                                        let (d1, d2, d3) =
                                            (DartId(d0.0 + 1), DartId(d0.0 + 2), DartId(d0.0 + 3));
                                        cmap.insert_vertex(d0.into(), vertices[vids[0]]);
                                        cmap.insert_vertex(d1.into(), vertices[vids[1]]);
                                        cmap.insert_vertex(d2.into(), vertices[vids[2]]);
                                        cmap.insert_vertex(d3.into(), vertices[vids[3]]);

                                        cmap.one_link(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                                        cmap.one_link(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                                        cmap.one_link(d2, d3); // edge d2 links vertices vids[2] & vids[3]
                                        cmap.one_link(d3, d0); // edge d3 links vertices vids[3] & vids[0]

                                        // record a trace of the built cell for future 2-sew
                                        sew_buffer.insert((vids[0], vids[1]), d0);
                                        sew_buffer.insert((vids[1], vids[2]), d1);
                                        sew_buffer.insert((vids[2], vids[3]), d2);
                                        sew_buffer.insert((vids[3], vids[0]), d3);
                                        Ok(())
                                    }
                                    _ => Err(BuilderError::UnsupportedVtkData(
                                        "CellType not supported in 2-maps",
                                    )),
                                });
                        if let Some(is_err) = errs.find(Result::is_err) {
                            return Err(is_err.unwrap_err()); // unwrap & wrap because type inference is clunky
                        }
                    }
                    VertexNumbers::XML { .. } => {
                        return Err(BuilderError::UnsupportedVtkData("XML format"));
                    }
                }
                Ok(())
            });
            // return the first error if there is one
            if let Some(is_err) = tmp.find(Result::is_err) {
                return Err(is_err.unwrap_err()); // unwrap & wrap because type inference is clunky
            }
        }
    }
    while let Some(((id0, id1), dart_id0)) = sew_buffer.pop_first() {
        if let Some(dart_id1) = sew_buffer.remove(&(id1, id0)) {
            cmap.two_sew(dart_id0, dart_id1);
        }
    }
    Ok(cmap)
}

// ------ TESTS
#[cfg(test)]
mod tests;
