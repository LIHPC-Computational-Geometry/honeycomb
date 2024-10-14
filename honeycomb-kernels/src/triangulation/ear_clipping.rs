use crate::triangulation::{
    check_requirements, crossp_from_verts, fetch_face_vertices, TriangulateError,
};
use honeycomb_core::cmap::{CMap2, DartIdentifier, FaceIdentifier, Orbit2, OrbitPolicy};
use honeycomb_core::geometry::CoordsFloat;

#[allow(clippy::missing_panics_doc)]
/// Triangulates a face using the ear clipping method.
///
/// This function triangulates a cell (face) of a 2D combinatorial map by iteratively
/// clipping ears (triangles) from the polygon until only a single triangle remains.
///
/// Note that:
/// - the function assumes that the polygon is simple (no self-intersections or holes) and that the
///   interior is lcoated on the LEFT SIDE of the cross-product.
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
/// the face satisfies requirements mentionned above because of the [Two ears theorem][TET].
///
/// [TET]: https://en.wikipedia.org/wiki/Two_ears_theorem
pub fn process_cell<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    face_id: FaceIdentifier,
    new_darts: &[DartIdentifier],
) -> Result<(), TriangulateError> {
    // fetch darts using a custom orbit so that they're ordered
    let mut darts: Vec<_> =
        Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), face_id as DartIdentifier).collect();
    let mut n = darts.len();

    // early checks - check # of darts & face size
    check_requirements(n, new_darts.len())?;

    // get associated vertices - check for undefined vertices
    let mut vertices = fetch_face_vertices(cmap, &darts)?;

    let mut ndart_id = new_darts[0];
    while n > 3 {
        let Some(ear) = (0..n).find(|idx| {
            // we're checking whether ABC is an ear or not
            let v1 = &vertices[*idx]; // A
            let v2 = &vertices[(*idx + 1) % n]; // B
            let v3 = &vertices[(*idx + 2) % n]; // C

            // we assume the interior of the polygon is on the left side
            let is_inside = {
                let tmp = crossp_from_verts(v1, v2, v3);
                tmp > T::epsilon()
            };

            let no_overlap = vertices
                .iter()
                .filter(|v| (**v != *v1) && (**v != *v2) && (**v != *v3))
                .all(|v| {
                    let sig12v = crossp_from_verts(v1, v2, v);
                    let sig23v = crossp_from_verts(v2, v3, v);
                    let sig31v = crossp_from_verts(v3, v1, v);

                    let has_pos =
                        (sig12v > T::zero()) || (sig23v > T::zero()) || (sig31v > T::zero());
                    let has_neg =
                        (sig12v < T::zero()) || (sig23v < T::zero()) || (sig31v < T::zero());

                    has_pos && has_neg
                });
            is_inside && no_overlap
        }) else {
            // println!("W: could not find ear to triangulate cell - skipping face {face_id}");
            return Err(TriangulateError::NoEar);
        };

        // edit cell; we use the nd1/nd2 edge to create a triangle from the ear
        // nd1 is on the side of the tri, nd2 on the side of the rest of the cell
        let d_ear1 = darts[ear];
        let d_ear2 = darts[(ear + 1) % n];
        let b0_d_ear1 = cmap.beta::<0>(d_ear1);
        let b1_d_ear2 = cmap.beta::<1>(d_ear2);
        let nd1 = ndart_id;
        let nd2 = ndart_id + 1;
        ndart_id += 2;
        // FIXME: using link methods only works if new identifiers are greater than all existing
        // FIXME: using sew methods in parallel could crash bc of the panic when no vertex defined
        cmap.one_unlink(b0_d_ear1);
        cmap.one_unlink(d_ear2);
        cmap.one_link(d_ear2, nd1);
        cmap.one_link(nd1, d_ear1);
        cmap.one_link(b0_d_ear1, nd2);
        cmap.one_link(nd2, b1_d_ear2);
        cmap.two_link(nd1, nd2);

        // edit existing vectors
        darts.remove((ear + 1) % n);
        darts.push(nd2);
        darts.swap_remove(ear);
        vertices.remove((ear + 1) % n);

        // update n
        n = Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), nd2).count();
    }

    Ok(())
}
