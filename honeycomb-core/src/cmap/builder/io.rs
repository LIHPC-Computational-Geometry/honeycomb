// TODO: replace this with a hashmap too
use std::collections::BTreeMap;

use itertools::multizip;
use num_traits::Zero;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use vtkio::model::{CellType, DataSet, VertexNumbers};
use vtkio::{IOBuffer, Vtk};

use crate::attributes::AttrStorageManager;
use crate::cmap::{BuilderError, CMap2, CMap3, DartIdType, VertexIdType};
use crate::geometry::{CoordsFloat, Vertex2, Vertex3};

// --- Custom

pub(crate) struct CMapFile {
    pub meta: (String, usize, usize),
    pub betas: String,
    pub unused: Option<String>,
    pub vertices: Option<String>,
}

pub(crate) fn parse_meta(meta_line: &str) -> Result<(String, usize, usize), BuilderError> {
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
        let mut sections = HashMap::default();
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
    if f.meta.1 != 2 {
        // mismatched dim
        return Err(BuilderError::BadMetaData(
            "mismatch between requested dimension and header",
        ));
    }
    let map = CMap2::new_with_undefined_attributes(f.meta.2, manager);

    // putting it in a scope to drop the data
    let betas = f.betas.lines().collect::<Vec<_>>();
    if betas.len() != 3 {
        // mismatched dim
        return Err(BuilderError::InconsistentData(
            "wrong number of beta functions",
        ));
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
        return Err(BuilderError::InconsistentData(
            "wrong number of values for the beta 0 function",
        ));
    }
    if b1.len() != f.meta.2 + 1 {
        return Err(BuilderError::InconsistentData(
            "wrong number of values for the beta 1 function",
        ));
    }
    if b2.len() != f.meta.2 + 1 {
        return Err(BuilderError::InconsistentData(
            "wrong number of values for the beta 2 function",
        ));
    }

    for (d, b0d, b1d, b2d) in multizip((
        (1..=f.meta.2),
        b0.into_iter().skip(1),
        b1.into_iter().skip(1),
        b2.into_iter().skip(1),
    )) {
        let b0d = b0d.map_err(|_| BuilderError::BadValue("could not parse a b0 value"))?;
        let b1d = b1d.map_err(|_| BuilderError::BadValue("could not parse a b1 value"))?;
        let b2d = b2d.map_err(|_| BuilderError::BadValue("could not parse a b2 value"))?;
        map.set_betas(d as DartIdType, [b0d, b1d, b2d]);
    }

    if let Some(unused) = f.unused {
        for u in unused.split_whitespace() {
            let d = u
                .parse()
                .map_err(|_| BuilderError::BadValue("could not parse an unused ID"))?;
            map.release_dart(d)
                .expect("E: unused dart has non-null beta images");
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
            map.set_vertex(id, Vertex2(T::from(x).unwrap(), T::from(y).unwrap()));
        }
    }

    Ok(map)
}

#[allow(clippy::too_many_lines)]
pub fn build_3d_from_cmap_file<T: CoordsFloat>(
    f: CMapFile,
    manager: AttrStorageManager, // FIXME: find a cleaner solution to populate the manager
) -> Result<CMap3<T>, BuilderError> {
    if f.meta.1 != 3 {
        // mismatched dim
        return Err(BuilderError::BadMetaData(
            "mismatch between requested dimension and header",
        ));
    }
    let map = CMap3::new_with_undefined_attributes(f.meta.2, manager);

    // putting it in a scope to drop the data
    let betas = f.betas.lines().collect::<Vec<_>>();
    if betas.len() != 4 {
        // mismatched dim
        return Err(BuilderError::InconsistentData(
            "wrong number of beta functions",
        ));
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
    let b3 = betas[3]
        .split_whitespace()
        .map(str::parse)
        .collect::<Vec<_>>();

    // mismatched dart number
    if b0.len() != f.meta.2 + 1 {
        return Err(BuilderError::InconsistentData(
            "wrong number of values for the beta 0 function",
        ));
    }
    if b1.len() != f.meta.2 + 1 {
        return Err(BuilderError::InconsistentData(
            "wrong number of values for the beta 1 function",
        ));
    }
    if b2.len() != f.meta.2 + 1 {
        return Err(BuilderError::InconsistentData(
            "wrong number of values for the beta 2 function",
        ));
    }
    if b3.len() != f.meta.2 + 1 {
        return Err(BuilderError::InconsistentData(
            "wrong number of values for the beta 2 function",
        ));
    }

    for (d, b0d, b1d, b2d, b3d) in multizip((
        (1..=f.meta.2),
        b0.into_iter().skip(1),
        b1.into_iter().skip(1),
        b2.into_iter().skip(1),
        b3.into_iter().skip(1),
    )) {
        let b0d = b0d.map_err(|_| BuilderError::BadValue("could not parse a b0 value"))?;
        let b1d = b1d.map_err(|_| BuilderError::BadValue("could not parse a b1 value"))?;
        let b2d = b2d.map_err(|_| BuilderError::BadValue("could not parse a b2 value"))?;
        let b3d = b3d.map_err(|_| BuilderError::BadValue("could not parse a b3 value"))?;
        map.set_betas(d as DartIdType, [b0d, b1d, b2d, b3d]);
    }

    if let Some(unused) = f.unused {
        for u in unused.split_whitespace() {
            let d = u
                .parse()
                .map_err(|_| BuilderError::BadValue("could not parse an unused ID"))?;
            map.release_dart(d)
                .expect("E: unused dart has non-null beta images");
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
            let z: f64 = it
                .next()
                .ok_or(BuilderError::BadValue("incorrect vertex line format"))?
                .parse()
                .map_err(|_| BuilderError::BadValue("could not parse vertex z coordinate"))?;
            if it.next().is_some() {
                return Err(BuilderError::BadValue("incorrect vertex line format"));
            }
            map.set_vertex(
                id,
                Vertex3(
                    T::from(x).unwrap(),
                    T::from(y).unwrap(),
                    T::from(z).unwrap(),
                ),
            );
        }
    }

    Ok(map)
}

// --- Abaqus INP

/// Node identifier used by an Abaqus input.
type InpNodeId = usize;

/// Parsed nodes and C3D8 element connectivity from an Abaqus INP document.
struct InpFile {
    /// Node coordinates indexed by their identifier.
    nodes: HashMap<InpNodeId, [f64; 3]>,
    /// C3D8 elements stored in Abaqus local node order.
    elements: Vec<[InpNodeId; 8]>,
}

/// Section currently being read from an Abaqus INP document.
enum InpSection {
    /// Data outside a supported section.
    None,
    /// A `*NODE` section.
    Nodes,
    /// A supported `*ELEMENT` section.
    Elements,
}

impl TryFrom<&str> for InpFile {
    type Error = BuilderError;

    /// Parse the supported subset of Abaqus INP data.
    #[allow(clippy::too_many_lines)]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut nodes = HashMap::default();
        let mut elements = Vec::new();
        let mut element_ids = HashSet::<usize>::default();
        let mut section = InpSection::None;

        for raw_line in value.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("**") {
                continue;
            }

            if line.starts_with('*') {
                let mut header = line.split(',');
                let keyword = header.next().expect("E: non-empty header").trim();
                if keyword.eq_ignore_ascii_case("*NODE") {
                    section = InpSection::Nodes;
                } else if keyword.eq_ignore_ascii_case("*ELEMENT") {
                    let element_type = header.find_map(|parameter| {
                        let (name, value) = parameter.split_once('=')?;
                        name.trim()
                            .eq_ignore_ascii_case("TYPE")
                            .then(|| value.trim().to_ascii_uppercase())
                    });
                    let Some(element_type) = element_type else {
                        return Err(BuilderError::BadInpData(
                            "element section has no TYPE parameter",
                        ));
                    };
                    if !element_type.starts_with("C3D8") {
                        return Err(BuilderError::UnsupportedInpData(
                            "only C3D8 element types are supported",
                        ));
                    }
                    section = InpSection::Elements;
                } else {
                    section = InpSection::None;
                }
                continue;
            }

            let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
            match section {
                InpSection::None => {}
                InpSection::Nodes => {
                    if fields.len() != 4 || fields.iter().any(|field| field.is_empty()) {
                        return Err(BuilderError::BadInpData("invalid node record"));
                    }
                    let node_id = fields[0]
                        .parse()
                        .map_err(|_| BuilderError::BadInpData("invalid node identifier"))?;
                    let coordinates = [
                        fields[1]
                            .parse()
                            .map_err(|_| BuilderError::BadInpData("invalid node coordinate"))?,
                        fields[2]
                            .parse()
                            .map_err(|_| BuilderError::BadInpData("invalid node coordinate"))?,
                        fields[3]
                            .parse()
                            .map_err(|_| BuilderError::BadInpData("invalid node coordinate"))?,
                    ];
                    if nodes.insert(node_id, coordinates).is_some() {
                        return Err(BuilderError::BadInpData("duplicate node identifier"));
                    }
                }
                InpSection::Elements => {
                    if fields.len() != 9 || fields.iter().any(|field| field.is_empty()) {
                        return Err(BuilderError::BadInpData("invalid C3D8 element record"));
                    }
                    let element_id = fields[0]
                        .parse()
                        .map_err(|_| BuilderError::BadInpData("invalid element identifier"))?;
                    if !element_ids.insert(element_id) {
                        return Err(BuilderError::BadInpData("duplicate element identifier"));
                    }
                    let mut element = [0; 8];
                    for (node_id, field) in element.iter_mut().zip(fields.iter().skip(1)) {
                        *node_id = field.parse().map_err(|_| {
                            BuilderError::BadInpData("invalid element node identifier")
                        })?;
                    }
                    if element.into_iter().collect::<HashSet<_>>().len() != 8 {
                        return Err(BuilderError::BadInpData(
                            "C3D8 element contains duplicate nodes",
                        ));
                    }
                    elements.push(element);
                }
            }
        }

        if nodes.is_empty() {
            return Err(BuilderError::BadInpData("file contains no nodes"));
        }
        if elements.is_empty() {
            return Err(BuilderError::BadInpData("file contains no C3D8 elements"));
        }
        if elements
            .iter()
            .flatten()
            .any(|node_id| !nodes.contains_key(node_id))
        {
            return Err(BuilderError::BadInpData(
                "element references an undefined node",
            ));
        }

        Ok(Self { nodes, elements })
    }
}

/// Outward-oriented quadrilateral faces, expressed in Abaqus C3D8 local node order.
const HEX_FACES: [[usize; 4]; 6] = [
    [0, 1, 2, 3],
    [1, 0, 4, 5],
    [2, 1, 5, 6],
    [3, 2, 6, 7],
    [0, 3, 7, 4],
    [5, 4, 7, 6],
];

/// First dart of each face in the internal 24-dart hexahedron representation.
const HEX_FACE_DART_OFFSETS: [DartIdType; 6] = [0, 4, 8, 12, 16, 20];

/// Representative dart of each vertex in the internal hexahedron representation.
const HEX_VERTEX_DART_OFFSETS: [DartIdType; 8] = [0, 1, 2, 3, 6, 7, 11, 15];

/// Beta-2 image of each dart, expressed as an offset within its hexahedron.
const HEX_BETA_2_OFFSETS: [DartIdType; 24] = [
    4, 8, 12, 16, 0, 19, 20, 9, 1, 7, 23, 13, 2, 11, 22, 17, 3, 15, 21, 5, 6, 18, 14, 10,
];

/// Initialize the beta relations of one 24-dart hexahedron.
fn initialize_hex<T: CoordsFloat>(map: &CMap3<T>, first_dart: DartIdType) {
    for face_offset in HEX_FACE_DART_OFFSETS {
        for edge_offset in 0..4 {
            let dart = first_dart + face_offset + edge_offset;
            let previous = first_dart + face_offset + (edge_offset + 3) % 4;
            let next = first_dart + face_offset + (edge_offset + 1) % 4;
            let beta_2 = first_dart + HEX_BETA_2_OFFSETS[(face_offset + edge_offset) as usize];
            map.set_betas(dart, [previous, next, beta_2, 0]);
        }
    }
}

/// Face waiting to be matched with an adjacent element.
#[derive(Clone, Copy)]
struct PendingFace {
    /// Face nodes in their oriented local order.
    nodes: [InpNodeId; 4],
    /// First dart of the corresponding quadrilateral face.
    first_dart: DartIdType,
}

/// Current sewing state of a face shared by mesh elements.
enum FaceState {
    /// The first occurrence of a face.
    Pending(PendingFace),
    /// A face already shared by two elements.
    Sewn,
}

/// Return an orientation-independent key for a quadrilateral face.
fn face_key(mut nodes: [InpNodeId; 4]) -> [InpNodeId; 4] {
    nodes.sort_unstable();
    nodes
}

/// Build a 3-map from the mesh contained in an Abaqus INP document.
pub(crate) fn build_3d_from_inp<T: CoordsFloat>(
    content: &str,
    manager: AttrStorageManager,
) -> Result<CMap3<T>, BuilderError> {
    let input = InpFile::try_from(content)?;
    let n_darts = input
        .elements
        .len()
        .checked_mul(24)
        .filter(|count| *count <= DartIdType::MAX as usize)
        .ok_or(BuilderError::BadInpData("mesh contains too many elements"))?;
    let map = CMap3::new_with_undefined_attributes(n_darts, manager);
    let mut faces = HashMap::<[InpNodeId; 4], FaceState>::default();

    for (element_index, element) in input.elements.iter().enumerate() {
        let first_dart = 1 + DartIdType::try_from(element_index * 24)
            .map_err(|_| BuilderError::BadInpData("mesh contains too many elements"))?;
        initialize_hex(&map, first_dart);

        for (face, &dart_offset) in HEX_FACES.iter().zip(&HEX_FACE_DART_OFFSETS) {
            let ordered_nodes = face.map(|index| element[index]);
            let key = face_key(ordered_nodes);
            match faces.get_mut(&key) {
                None => {
                    faces.insert(
                        key,
                        FaceState::Pending(PendingFace {
                            nodes: ordered_nodes,
                            first_dart: first_dart + dart_offset,
                        }),
                    );
                }
                Some(state @ FaceState::Pending(_)) => {
                    let FaceState::Pending(previous) = *state else {
                        unreachable!()
                    };
                    let Some(reverse_edge_offset) = (0..4).find(|&index| {
                        ordered_nodes[index] == previous.nodes[1]
                            && ordered_nodes[(index + 1) % 4] == previous.nodes[0]
                    }) else {
                        return Err(BuilderError::BadInpData(
                            "adjacent elements have inconsistent face orientations",
                        ));
                    };
                    map.link::<3>(
                        previous.first_dart,
                        first_dart + dart_offset + reverse_edge_offset as DartIdType,
                    )
                    .map_err(|_| BuilderError::BadInpData("could not link adjacent elements"))?;
                    *state = FaceState::Sewn;
                }
                Some(FaceState::Sewn) => {
                    return Err(BuilderError::BadInpData(
                        "face is shared by more than two elements",
                    ));
                }
            }
        }
    }

    // A node ID normally maps to one vertex orbit. If a deck reuses an ID across topologically
    // disconnected components, each orbit still needs its own copy of the coordinate.
    let mut vertex_nodes = HashMap::<VertexIdType, InpNodeId>::default();
    for (element_index, element) in input.elements.iter().enumerate() {
        let first_dart = 1 + element_index as DartIdType * 24;
        for (&node_id, &dart_offset) in element.iter().zip(&HEX_VERTEX_DART_OFFSETS) {
            let vertex_id = map.vertex_id(first_dart + dart_offset);
            if vertex_nodes
                .insert(vertex_id, node_id)
                .is_some_and(|previous| previous != node_id)
            {
                return Err(BuilderError::BadInpData(
                    "topology merges different node identifiers",
                ));
            }
        }
    }

    for (vertex_id, node_id) in vertex_nodes {
        let [x, y, z] = input.nodes[&node_id];
        let vertex = Vertex3(
            T::from(x).ok_or(BuilderError::BadInpData("node coordinate is out of range"))?,
            T::from(y).ok_or(BuilderError::BadInpData("node coordinate is out of range"))?,
            T::from(z).ok_or(BuilderError::BadInpData("node coordinate is out of range"))?,
        );
        map.set_vertex(vertex_id, vertex);
    }

    Ok(map)
}

// --- VTK

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

// ------ building routine

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
            return Err(BuilderError::UnsupportedVtkData("dataset not supported"));
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
                        ));
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
                                        let d0 = cmap.allocate_used_darts(3);
                                        let (d1, d2) = (d0 + 1, d0 + 2);
                                        cmap.set_vertex(d0 as VertexIdType, vertices[vids[0]]);
                                        cmap.set_vertex(d1 as VertexIdType, vertices[vids[1]]);
                                        cmap.set_vertex(d2 as VertexIdType, vertices[vids[2]]);
                                        cmap.link::<1>(d0, d1).unwrap(); // edge d0 links vertices vids[0] & vids[1]
                                        cmap.link::<1>(d1, d2).unwrap(); // edge d1 links vertices vids[1] & vids[2]
                                        cmap.link::<1>(d2, d0).unwrap(); // edge d2 links vertices vids[2] & vids[0]
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
                                        let d0 = cmap.allocate_used_darts(n_vertices);
                                        (0..n_vertices).for_each(|i| {
                                            let di = d0 + i as DartIdType;
                                            let dip1 =
                                                if i == n_vertices - 1 { d0 } else { di + 1 };
                                            cmap.set_vertex(di as VertexIdType, vertices[vids[i]]);
                                            cmap.link::<1>(di, dip1).unwrap();
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
                                        let d0 = cmap.allocate_used_darts(4);
                                        let (d1, d2, d3) = (d0 + 1, d0 + 2, d0 + 3);
                                        cmap.set_vertex(d0 as VertexIdType, vertices[vids[0]]);
                                        cmap.set_vertex(d1 as VertexIdType, vertices[vids[1]]);
                                        cmap.set_vertex(d2 as VertexIdType, vertices[vids[2]]);
                                        cmap.set_vertex(d3 as VertexIdType, vertices[vids[3]]);
                                        cmap.link::<1>(d0, d1).unwrap(); // edge d0 links vertices vids[0] & vids[1]
                                        cmap.link::<1>(d1, d2).unwrap(); // edge d1 links vertices vids[1] & vids[2]
                                        cmap.link::<1>(d2, d3).unwrap(); // edge d2 links vertices vids[2] & vids[3]
                                        cmap.link::<1>(d3, d0).unwrap(); // edge d3 links vertices vids[3] & vids[0]
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
            cmap.sew::<2>(dart_id0, dart_id1).unwrap();
        }
    }
    Ok(cmap)
}
