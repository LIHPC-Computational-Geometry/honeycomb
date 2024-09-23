use honeycomb_core::cmap::{CMap2, DartIdentifier, FaceIdentifier, Orbit2, OrbitPolicy};
use honeycomb_core::geometry::CoordsFloat;

pub fn process_cell<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    face_id: FaceIdentifier,
    new_darts: &[DartIdentifier],
) {
    let mut n = Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), face_id as DartIdentifier).count();

    // early rets
    if n <= 3 {
        println!("I: "); //TODO: complete
        return;
    }
    if (n - 3) * 2 != new_darts.len() {
        println!("W: "); //TODO: complete
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
