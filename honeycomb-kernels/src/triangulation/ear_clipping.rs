use honeycomb_core::cmap::{CMap2, DartIdentifier, FaceIdentifier, Orbit2, OrbitPolicy};
use honeycomb_core::geometry::CoordsFloat;

/// Triangulates a face using the ear clipping method.
///
/// This function triangulates a cell (face) of a 2D combinatorial map by iteratively
/// clipping ears (triangles) from the polygon until only a single triangle remains.
///
/// Note that:
/// - the function assumes that the polygon is simple (no self-intersections or holes).
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
/// # Panics
///
/// The function will panic if a dart of the face does not have an associated vertex.
pub fn process_cell<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    face_id: FaceIdentifier,
    new_darts: &[DartIdentifier],
) {
    let mut n = Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), face_id as DartIdentifier).count();

    // early rets
    if n <= 3 {
        println!("I: face {face_id} is an {n}-gon -- skipping triangulation");
        return;
    }
    if (n - 3) * 2 != new_darts.len() {
        println!("W: not enough pre-allocated darts to triangulate face {face_id} -- skipping triangulation");
        return;
    }

    let mut darts: Vec<_> =
        Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), face_id as DartIdentifier).collect();
    let mut vertices: Vec<_> = darts
        .iter()
        .map(|dart_id| {
            cmap.vertex(cmap.vertex_id(*dart_id))
                .expect("E: found a topological vertex with no associated coordinates")
        })
        .collect();
    let mut ndart_id = new_darts[0];
    while n > 3 {
        let ear = (0..n)
            .find(|idx| {
                let v1 = vertices[*idx];
                let v2 = vertices[(*idx + 1) % n];
                let v3 = vertices[(*idx + 2) % n];
                let v4 = vertices[(*idx + 3) % n];
                // we can easily check if two edges make up an ear using orientation
                let v_in = v3 - v2; // BC
                let v_out = v4 - v3; // CD
                let v_new = v1 - v3; // CA
                let or1 = (v_in.x() * v_new.y() - v_new.x() * v_in.y()).signum(); // in ^ new
                let or2 = (-v_new.x() * v_out.y() + v_out.x() * v_new.y()).signum(); // -new ^ out
                or1 == or2
            })
            .unwrap(); // safe unwrap bc of the two ear theorem

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
        darts.remove(ear + 1);
        darts.push(nd2);
        darts.swap_remove(ear);
        vertices.remove(ear + 1);

        // update n
        n = Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), nd2).count();
    }
}
