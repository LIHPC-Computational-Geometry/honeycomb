use crate::prelude::{BuilderError, CMap2, CMapBuilder, DartIdType, Vertex2, VertexIdType};
use crate::{attributes::AttrStorageManager, geometry::CoordsFloat};

use std::collections::{BTreeMap, HashMap};

use itertools::multizip;
use num_traits::Zero;
use vtkio::model::{CellType, DataSet, VertexNumbers};
use vtkio::{IOBuffer, Vtk};

// --- Custom

pub(crate) struct CMapFile {
    pub meta: (String, usize, usize),
    pub betas: String,
    pub unused: Option<String>,
    pub vertices: Option<String>,
}

fn parse_meta(meta_line: &str) -> Result<(String, usize, usize), BuilderError> {
    let parts: Vec<&str> = meta_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(BuilderError::BadMetaData("incorrect format"));
    }

    Ok((
        parts[0].to_string(),
        parts[1]
            .parse()
            .map_err(|_| BuilderError::BadMetaData("could not parse dimension"))?,
        parts[2]
            .parse()
            .map_err(|_| BuilderError::BadMetaData("could not parse dart number"))?,
    ))
}

impl TryFrom<String> for CMapFile {
    type Error = BuilderError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut sections = HashMap::new();
        let mut current_section = String::new();

        for line in value.trim().lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                // ignore empty & comment lines
                continue;
            }
            if trimmed.starts_with('[') && trimmed.contains(']') {
                // process section header
                let section_name = trimmed.trim_matches(['[', ']']).to_lowercase();

                if section_name != "meta"
                    && section_name != "betas"
                    && section_name != "unused"
                    && section_name != "vertices"
                {
                    return Err(BuilderError::UnknownHeader(section_name));
                }

                if sections
                    .insert(section_name.clone(), String::new())
                    .is_some()
                {
                    return Err(BuilderError::DuplicatedSection(section_name));
                }
                current_section = section_name;

                continue;
            }
            if !current_section.is_empty() {
                // regular line
                let line_without_comment = trimmed.split('#').next().unwrap().trim();
                if !line_without_comment.is_empty() {
                    let current_content = sections.get_mut(&current_section).unwrap();
                    if !current_content.is_empty() {
                        current_content.push('\n');
                    }
                    current_content.push_str(line_without_comment);
                }
            }
        }

        if !sections.contains_key("meta") {
            // missing required section
            return Err(BuilderError::MissingSection("meta"));
        }
        if !sections.contains_key("betas") {
            // missing required section
            return Err(BuilderError::MissingSection("betas"));
        }

        Ok(Self {
            meta: parse_meta(sections["meta"].as_str())?,
            betas: sections["betas"].clone(),
            unused: sections.get("unused").cloned(),
            vertices: sections.get("vertices").cloned(),
        })
    }
}

// ------ building routines

pub fn build_2d_from_cmap_file<T: CoordsFloat>(
    f: CMapFile,
    manager: AttrStorageManager, // FIXME: find a cleaner solution to populate the manager
) -> Result<CMap2<T>, BuilderError> {
    let mut map = CMap2::new_with_undefined_attributes(f.meta.2, manager);

    // putting it in a scope to drop the data
    let betas = f.betas.lines().collect::<Vec<_>>();
    if betas.len() != 3 {
        // mismatched dim
        todo!()
    }
    let b0 = betas[0]
        .split_whitespace()
        .map(str::parse)
        .collect::<Vec<_>>();
    let b1 = betas[1]
        .split_whitespace()
        .map(str::parse)
        .collect::<Vec<_>>();
    let b2 = betas[2]
        .split_whitespace()
        .map(str::parse)
        .collect::<Vec<_>>();

    // mismatched dart number
    if b0.len() != f.meta.2 + 1 {
        todo!()
    }
    if b1.len() != f.meta.2 + 1 {
        todo!()
    }
    if b2.len() != f.meta.2 + 1 {
        todo!()
    }

    for (d, b0d, b1d, b2d) in multizip((
        (1..=f.meta.2),
        b0.into_iter().skip(1),
        b1.into_iter().skip(1),
        b2.into_iter().skip(1),
    )) {
        let b0d = b0d.map_err(|_| todo!())?;
        let b1d = b1d.map_err(|_| todo!())?;
        let b2d = b2d.map_err(|_| todo!())?;
        map.set_betas(d as DartIdType, [b0d, b1d, b2d]);
    }

    if let Some(unused) = f.unused {
        for u in unused.split_whitespace() {
            let d = u.parse().map_err(|_| todo!())?;
            map.remove_free_dart(d);
        }
    }

    if let Some(vertices) = f.vertices {
        for l in vertices.trim().lines() {
            let mut it = l.split_whitespace();
            let id: VertexIdType = it
                .next()
                .ok_or(BuilderError::BadValue("incorrect vertex line format"))?
                .parse()
                .map_err(|_| BuilderError::BadValue("could not parse vertex ID"))?;
            let x: f64 = it
                .next()
                .ok_or(BuilderError::BadValue("incorrect vertex line format"))?
                .parse()
                .map_err(|_| BuilderError::BadValue("could not parse vertex x coordinate"))?;
            let y: f64 = it
                .next()
                .ok_or(BuilderError::BadValue("incorrect vertex line format"))?
                .parse()
                .map_err(|_| BuilderError::BadValue("could not parse vertex y coordinate"))?;
            if it.next().is_some() {
                return Err(BuilderError::BadValue("incorrect vertex line format"));
            }
            map.force_write_vertex(id, (T::from(x).unwrap(), T::from(y).unwrap()));
        }
    }

    Ok(map)
}

// --- VTK

/// Create a [`CMapBuilder`] from the VTK file specified by the path.
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

// ------ building routine

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
    let mut sew_buffer: BTreeMap<(usize, usize), DartIdType> = BTreeMap::new();
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
                                        let (d1, d2) = (d0 + 1, d0 + 2);
                                        cmap.force_write_vertex(
                                            d0 as VertexIdType,
                                            vertices[vids[0]],
                                        );
                                        cmap.force_write_vertex(
                                            d1 as VertexIdType,
                                            vertices[vids[1]],
                                        );
                                        cmap.force_write_vertex(
                                            d2 as VertexIdType,
                                            vertices[vids[2]],
                                        );
                                        cmap.force_link::<1>(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                                        cmap.force_link::<1>(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                                        cmap.force_link::<1>(d2, d0); // edge d2 links vertices vids[2] & vids[0]
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
                                            let di = d0 + i as DartIdType;
                                            let dip1 =
                                                if i == n_vertices - 1 { d0 } else { di + 1 };
                                            cmap.force_write_vertex(
                                                di as VertexIdType,
                                                vertices[vids[i]],
                                            );
                                            cmap.force_link::<1>(di, dip1);
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
                                        let (d1, d2, d3) = (d0 + 1, d0 + 2, d0 + 3);
                                        cmap.force_write_vertex(
                                            d0 as VertexIdType,
                                            vertices[vids[0]],
                                        );
                                        cmap.force_write_vertex(
                                            d1 as VertexIdType,
                                            vertices[vids[1]],
                                        );
                                        cmap.force_write_vertex(
                                            d2 as VertexIdType,
                                            vertices[vids[2]],
                                        );
                                        cmap.force_write_vertex(
                                            d3 as VertexIdType,
                                            vertices[vids[3]],
                                        );
                                        cmap.force_link::<1>(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                                        cmap.force_link::<1>(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                                        cmap.force_link::<1>(d2, d3); // edge d2 links vertices vids[2] & vids[3]
                                        cmap.force_link::<1>(d3, d0); // edge d3 links vertices vids[3] & vids[0]
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
            cmap.force_sew::<2>(dart_id0, dart_id1);
        }
    }
    Ok(cmap)
}
