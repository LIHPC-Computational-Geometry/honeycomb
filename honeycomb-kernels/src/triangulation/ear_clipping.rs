use honeycomb_core::cmap::{CMap2, DartIdType, FaceIdType, OrbitPolicy};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};
use honeycomb_core::stm::{Transaction, TransactionClosureResult, abort, try_or_coerce};
use smallvec::SmallVec;

use crate::triangulation::{TriangulateError, check_requirements};

#[allow(clippy::missing_panics_doc)]
/// Triangulates a face using the ear clipping method.
///
/// This function triangulates a cell (face) of a 2D combinatorial map by iteratively
/// clipping ears (triangles) from the polygon until only a single triangle remains.
///
/// Note that:
/// - the function assumes that the polygon is simple (no self-intersections or holes) and that the
///   interior is located on the LEFT SIDE of the cross-product, i.e. oriented counter-clockwise.
/// - the implementation might need adjustments for parallel operations or when working with
///   identifiers not greater than existing ones.
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
/// - It ensures that the number of new darts provided matches the triangulation requirement.
/// - Iteratively finds an 'ear' of the polygon, where an ear is defined by three consecutive
///   vertices where the triangle formed by these vertices is inside the original polygon.
/// - Clips this ear by updating the combinatorial map's structure, effectively removing one
///   vertex from the polygon per iteration.
/// - Updates the list of darts and vertices accordingly after each ear clip.
/// - Continues until the polygon is reduced to a triangle.
///
/// # Errors
///
/// This function will return an error if the face wasn't triangulated. There can be multiple
/// reason for this:
/// - The face is incompatible with the operation (made of 1, 2 or 3 vertices)
/// - The number of pre-allocated darts did not match the expected number (see [#arguments])
/// - The face contains one or more undefined vertices
/// - The face does not contain an ear
///
/// Note that in all cases except the last, the face will remain the same as it was before the
/// function call.
///
/// Because the ear clipping algorithm works iteratively, the face may be modified a few times
/// before reaching a state where no ear can be found. Note, however, that this cannot occur if
/// the face satisfies requirements mentioned above because of the [Two ears theorem][TET].
///
/// [TET]: https://en.wikipedia.org/wiki/Two_ears_theorem
pub fn earclip_cell_countercw<T: CoordsFloat>(
    t: &mut Transaction,
    cmap: &CMap2<T>,
    face_id: FaceIdType,
    new_darts: &[DartIdType],
) -> TransactionClosureResult<(), TriangulateError> {
    process_cell(t, cmap, face_id, new_darts, |v1, v2, v3| {
        Vertex2::cross_product_from_vertices(v1, v2, v3) > T::zero()
    })
}

#[allow(clippy::missing_panics_doc)]
/// Triangulates a face using the ear clipping method.
///
/// This function triangulates a cell (face) of a 2D combinatorial map by iteratively
/// clipping ears (triangles) from the polygon until only a single triangle remains.
///
/// Note that:
/// - the function assumes that the polygon is simple (no self-intersections or holes) and that the
///   interior is located on the RIGHT SIDE of the cross-product, i.e. oriented clockwise.
/// - the implementation might need adjustments for parallel operations or when working with
///   identifiers not greater than existing ones.
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
/// - It ensures that the number of new darts provided matches the triangulation requirement.
/// - Iteratively finds an 'ear' of the polygon, where an ear is defined by three consecutive
///   vertices where the triangle formed by these vertices is inside the original polygon.
/// - Clips this ear by updating the combinatorial map's structure, effectively removing one
///   vertex from the polygon per iteration.
/// - Updates the list of darts and vertices accordingly after each ear clip.
/// - Continues until the polygon is reduced to a triangle.
///
/// # Errors
///
/// This function will return an error if the face wasn't triangulated. There can be multiple
/// reason for this:
/// - The face is incompatible with the operation (made of 1, 2 or 3 vertices)
/// - The number of pre-allocated darts did not match the expected number (see [#arguments])
/// - The face contains one or more undefined vertices
/// - The face does not contain an ear
///
/// Note that in all cases except the last, the face will remain the same as it was before the
/// function call.
///
/// Because the ear clipping algorithm works iteratively, the face may be modified a few times
/// before reaching a state where no ear can be found. Note, however, that this cannot occur if
/// the face satisfies requirements mentioned above because of the [Two ears theorem][TET].
///
/// [TET]: https://en.wikipedia.org/wiki/Two_ears_theorem
pub fn earclip_cell_cw<T: CoordsFloat>(
    t: &mut Transaction,
    cmap: &CMap2<T>,
    face_id: FaceIdType,
    new_darts: &[DartIdType],
) -> TransactionClosureResult<(), TriangulateError> {
    process_cell(t, cmap, face_id, new_darts, |v1, v2, v3| {
        Vertex2::cross_product_from_vertices(v1, v2, v3) < T::zero()
    })
}

// -- internals

fn process_cell<T: CoordsFloat>(
    t: &mut Transaction,
    cmap: &CMap2<T>,
    face_id: FaceIdType,
    new_darts: &[DartIdType],
    is_inside_fn: impl FnOnce(&Vertex2<T>, &Vertex2<T>, &Vertex2<T>) -> bool + Copy,
) -> TransactionClosureResult<(), TriangulateError> {
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

    // early checks - check # of darts & face size
    if let Err(e) = check_requirements(darts.len(), new_darts.len()) {
        abort(e)?;
    }
    let mut darts = darts.clone();
    let mut vertices = vertices.clone();
    let mut n = darts.len();
    for sl in new_darts.chunks_exact(2) {
        let &[nd1, nd2] = sl else { unreachable!() };

        let Some(ear) = (0..n).find(|idx| {
            // we're checking whether ABC is an ear or not
            let v1 = &vertices[*idx]; // A
            let v2 = &vertices[(*idx + 1) % n]; // B
            let v3 = &vertices[(*idx + 2) % n]; // C

            // we assume the interior of the polygon is on the left side
            let is_inside = is_inside_fn(v1, v2, v3);

            let no_overlap = vertices
                .iter()
                .filter(|v| (**v != *v1) && (**v != *v2) && (**v != *v3))
                .all(|v| {
                    let sig12v = Vertex2::cross_product_from_vertices(v1, v2, v);
                    let sig23v = Vertex2::cross_product_from_vertices(v2, v3, v);
                    let sig31v = Vertex2::cross_product_from_vertices(v3, v1, v);

                    let has_pos =
                        (sig12v > T::zero()) || (sig23v > T::zero()) || (sig31v > T::zero());
                    let has_neg =
                        (sig12v < T::zero()) || (sig23v < T::zero()) || (sig31v < T::zero());

                    has_pos && has_neg
                });
            is_inside && no_overlap
        }) else {
            abort(TriangulateError::NoEar)?
        };

        // edit cell; we use the nd1/nd2 edge to create a triangle from the ear
        // nd1 is on the side of the tri, nd2 on the side of the rest of the cell
        let d_ear1 = darts[ear];
        let d_ear2 = darts[(ear + 1) % n];
        let b0_d_ear1 = cmap.beta_transac::<0>(t, d_ear1)?;
        let b1_d_ear2 = cmap.beta_transac::<1>(t, d_ear2)?;
        try_or_coerce!(cmap.unsew::<1>(t, b0_d_ear1), TriangulateError);
        try_or_coerce!(cmap.unsew::<1>(t, d_ear2), TriangulateError);
        try_or_coerce!(cmap.sew::<1>(t, d_ear2, nd1), TriangulateError);
        try_or_coerce!(cmap.sew::<1>(t, nd1, d_ear1), TriangulateError);
        try_or_coerce!(cmap.sew::<1>(t, b0_d_ear1, nd2), TriangulateError);
        try_or_coerce!(cmap.sew::<1>(t, nd2, b1_d_ear2), TriangulateError);
        try_or_coerce!(cmap.sew::<2>(t, nd1, nd2), TriangulateError);

        // edit existing vectors
        darts.remove((ear + 1) % n);
        darts.push(nd2);
        darts.swap_remove(ear);
        vertices.remove((ear + 1) % n);

        // update n
        n -= 1;
    }

    // Given error checking inside the `for` and the check on darts at the beginning,
    // this should ALWAYS be verified.
    assert_eq!(n, 3, "E: unreachable");

    Ok(())
}
