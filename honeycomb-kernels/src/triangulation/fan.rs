use honeycomb_core::cmap::{CMap2, DartIdType, FaceIdType, OrbitPolicy};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};
use honeycomb_core::stm::{Transaction, TransactionClosureResult, abort, try_or_coerce};
use smallvec::SmallVec;

use crate::triangulation::{TriangulateError, check_requirements};

#[allow(clippy::missing_panics_doc)]
/// Triangulates a face using a fan triangulation method.
///
/// This function triangulates a cell (face) in a 2D combinatorial map by creating a fan of
/// triangles from a chosen vertex to all other vertices of the polygon, if such a vertex exist.
///
/// Note that this function will not have any effect if the polygon isn't fannable.
///
/// # Arguments
///
/// - `cmap: &mut CMap2` - A mutable reference to the modified `CMap2`.
/// - `face_id: FaceIdentifier` - Identifier of the face to triangulate within the map.
/// - `new_darts: &[DartIdentifier]` - Identifiers of pre-allocated darts for the new edges;
///   the slice length should match the expected number of edges created by the triangulation. For
///   an `n`-sided polygon, the number of created edge is `n-3`, so the number of dart is `(n-3)*2`.
///
/// # Behavior
///
/// - The function begins by checking if the face has 3 or fewer vertices, in which case
///   it's already triangulated or cannot be further processed.
/// - It verifies if the number of new darts matches the expected number for triangulation.
/// - The function then attempts to find a vertex from which all other vertices can be seen
///   (a star vertex), using the orientation properties of N-maps.
/// - If such a star vertex is found, the function proceeds to create triangles by linking
///   new darts in a fan-like structure from this vertex. Otherwise, the cell is left unchanged
///
/// # Errors
///
/// This function will return an error if the face wasn't triangulated. There can be multiple
/// reason for this:
/// - The face is incompatible with the operation (made of 1, 2 or 3 vertices)
/// - The number of pre-allocated darts did not match the expected number (see [#arguments])
/// - The face contains one or more undefined vertices
/// - The face isn't starrable
///
/// Note that in any of these cases, the face will remain the same as it was before the function
/// call.
pub fn process_cell<T: CoordsFloat>(
    t: &mut Transaction,
    cmap: &CMap2<T>,
    face_id: FaceIdType,
    new_darts: &[DartIdType],
) -> TransactionClosureResult<(), TriangulateError> {
    // fetch darts using a custom orbit so that they're ordered
    let mut darts: SmallVec<DartIdType, 16> = SmallVec::new();
    let mut vertices: SmallVec<Vertex2<T>, 16> = SmallVec::new();

    for d in cmap.orbit_transac(t, OrbitPolicy::FaceLinear, face_id as DartIdType) {
        darts.push(d?);
    }
    for &d in &darts {
        let vid = cmap.vertex_id_transac(t, d)?;
        let v = if let Some(val) = cmap.read_vertex(t, vid)? {
            val
        } else {
            abort(TriangulateError::UndefinedFace(
                "one or more undefined vertices",
            ))?
        };
        vertices.push(v);
    }
    let n = darts.len();
    // early checks - check # of darts & face size
    if let Err(e) = check_requirements(n, new_darts.len()) {
        abort(e)?;
    }

    // iterating by ref so that we can still access the list
    let star = darts
        .iter()
        .zip(vertices.iter())
        .enumerate()
        .find_map(|(id, (d0, v0))| {
            let mut tmp = vertices
                .windows(2)
                .enumerate()
                // remove segments directly attached to v0
                .filter(|(i_seg, _)| !((n + i_seg) % n == id || (n + i_seg - 1) % n == id))
                .map(|(_, val)| {
                    let [v1, v2] = val else { unreachable!() };
                    Vertex2::cross_product_from_vertices(v0, v1, v2)
                });
            let signum = tmp.next().map(T::signum).unwrap();
            for v in tmp {
                if v.signum() != signum || v.abs() < T::epsilon() {
                    return None;
                }
            }
            Some(d0)
        });

    if let Some(sdart) = star {
        // if we found a dart from the previous computations, it means the polygon is "fannable"
        // THIS CANNOT BE PARALLELIZED AS IS
        let b0_sdart = cmap.beta_transac::<0>(t, *sdart)?;
        let vid = cmap.vertex_id_transac(t, *sdart)?;
        let v0 = cmap.read_vertex(t, vid)?.unwrap();
        try_or_coerce!(cmap.unsew::<1>(t, b0_sdart), TriangulateError);
        let mut d0 = *sdart;
        for sl in new_darts.chunks_exact(2) {
            let [d1, d2] = sl else { unreachable!() };
            let b1_d0 = cmap.beta_transac::<1>(t, d0)?;
            let b1b1_d0 = cmap.beta_transac::<1>(t, b1_d0)?;
            try_or_coerce!(cmap.unsew::<1>(t, b1_d0), TriangulateError);
            try_or_coerce!(cmap.sew::<2>(t, *d1, *d2), TriangulateError);
            try_or_coerce!(cmap.sew::<1>(t, *d2, b1b1_d0), TriangulateError);
            try_or_coerce!(cmap.sew::<1>(t, b1_d0, *d1), TriangulateError);
            try_or_coerce!(cmap.sew::<1>(t, *d1, d0), TriangulateError);
            d0 = *d2;
        }
        let b1_d0 = cmap.beta_transac::<1>(t, d0)?;
        let b1b1_d0 = cmap.beta_transac::<1>(t, b1_d0)?;
        try_or_coerce!(cmap.sew::<1>(t, b1b1_d0, d0), TriangulateError);
        let vid = cmap.vertex_id_transac(t, *sdart)?;
        cmap.write_vertex(t, vid, v0)?;
    } else {
        // println!("W: face {face_id} isn't fannable -- skipping triangulation");
        abort(TriangulateError::NonFannable)?;
    }

    Ok(())
}

#[allow(clippy::missing_panics_doc)]
/// Triangulates a face using a fan triangulation method.
///
/// This function triangulates a cell (face) in a 2D combinatorial map by creating a fan of
/// triangles from a the first vertex of
///
/// **Note that this function assumes the polygon is convex and correctly defined (i.e. all vertices
/// are) and may fail or produce incorrect results if called on a cell that does not verify these
/// requirements**.
///
/// # Arguments
///
/// - `cmap: &mut CMap2` - A mutable reference to the modified `CMap2`.
/// - `face_id: FaceIdentifier` - Identifier of the face to triangulate within the map.
/// - `new_darts: &[DartIdentifier]` - Identifiers of pre-allocated darts for the new edges;
///   the slice length should match the expected number of edges created by the triangulation. For
///   an `n`-sided polygon, the number of created edge is `n-3`, so the number of dart is `(n-3)*2`.
///
/// # Behavior
///
/// - The function begins by checking if the face has 3 or fewer vertices, in which case
///   it's already triangulated or cannot be further processed.
/// - It verifies if the number of new darts matches the expected number for triangulation.
/// - The function creates triangles by linking new darts in a fan-like structure to the first
///   vertex of the polygon. **This is done unconditionally, whether the polygon is convex or not**.
///
/// # Errors
///
/// This function will return an error if the face wasn't triangulated. There can be multiple
/// reason for this:
/// - The face is incompatible with the operation (made of 1, 2 or 3 vertices)
/// - The number of pre-allocated darts did not match the expected number (see [#arguments])
///
/// Note that in any of these cases, the face will remain the same as it was before the function
/// call.
pub fn process_convex_cell<T: CoordsFloat>(
    t: &mut Transaction,
    cmap: &CMap2<T>,
    face_id: FaceIdType,
    new_darts: &[DartIdType],
) -> TransactionClosureResult<(), TriangulateError> {
    // fetch darts using a custom orbit so that they're ordered
    let mut darts: SmallVec<DartIdType, 16> = SmallVec::new();

    for d in cmap.orbit_transac(t, OrbitPolicy::FaceLinear, face_id as DartIdType) {
        darts.push(d?);
    }
    let n = darts.len();

    // early checks - check # of darts & face size
    if let Err(e) = check_requirements(n, new_darts.len()) {
        abort(e)?;
    }

    // we assume the polygon is convex (== starrable from any vertex)
    let sdart = face_id as DartIdType;
    // THIS CANNOT BE PARALLELIZED AS IS
    let b0_sdart = cmap.beta_transac::<0>(t, sdart)?;
    let vid = cmap.vertex_id_transac(t, sdart)?;
    let v0 = cmap.read_vertex(t, vid)?.unwrap();
    try_or_coerce!(cmap.unsew::<1>(t, b0_sdart), TriangulateError);
    let mut d0 = sdart;
    for sl in new_darts.chunks_exact(2) {
        let [d1, d2] = sl else { unreachable!() };
        let b1_d0 = cmap.beta_transac::<1>(t, d0)?;
        let b1b1_d0 = cmap.beta_transac::<1>(t, b1_d0)?;
        try_or_coerce!(cmap.unsew::<1>(t, b1_d0), TriangulateError);
        try_or_coerce!(cmap.sew::<2>(t, *d1, *d2), TriangulateError);
        try_or_coerce!(cmap.sew::<1>(t, *d2, b1b1_d0), TriangulateError);
        try_or_coerce!(cmap.sew::<1>(t, b1_d0, *d1), TriangulateError);
        try_or_coerce!(cmap.sew::<1>(t, *d1, d0), TriangulateError);
        d0 = *d2;
    }
    let b1_d0 = cmap.beta_transac::<1>(t, d0)?;
    let b1b1_d0 = cmap.beta_transac::<1>(t, b1_d0)?;
    try_or_coerce!(cmap.sew::<1>(t, b1b1_d0, d0), TriangulateError);
    let vid = cmap.vertex_id_transac(t, sdart)?;
    cmap.write_vertex(t, vid, v0)?;

    Ok(())
}
