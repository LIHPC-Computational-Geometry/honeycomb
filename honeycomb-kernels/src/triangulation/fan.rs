use honeycomb_core::cmap::{CMap2, DartIdentifier, FaceIdentifier, Orbit2, OrbitPolicy};
use honeycomb_core::geometry::CoordsFloat;

pub fn process_cell<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    face_id: FaceIdentifier,
    new_darts: &[DartIdentifier],
) {
    // fetch darts using a custom orbit so that they're ordered
    let darts: Vec<_> =
        Orbit2::new(cmap, OrbitPolicy::Custom(&[1]), face_id as DartIdentifier).collect();
    let n = darts.len();

    // early rets
    if n == 3 {
        println!("I: "); //TODO: complete
        return;
    }
    if (n - 3) * 2 != new_darts.len() {
        println!("W: "); //TODO: complete
        return;
    }

    let vertices: Vec<_> = darts
        .iter()
        .map(|dart_id| {
            cmap.vertex(cmap.vertex_id(*dart_id))
                .expect("E: found a topological vertex with no associated coordinates")
        })
        .collect();

    // iterating by ref so that we can still access the list
    let star = darts
        .iter()
        .zip(vertices.iter())
        .enumerate()
        .find_map(|(id, (d0, v0))| {
            let mut tmp = vertices
                .windows(2)
                // remove segments directly attached to v0
                .filter(|_| !((n + id) % n == 0 || (n + id - 1) % n == 0))
                .map(|val| {
                    let [v1, v2] = val else { unreachable!() };
                    let vec_in = *v1 - *v0;
                    let vec_out = *v2 - *v1;
                    let cosine = vec_in.dot(&vec_out) / (vec_in.norm() * vec_out.norm());
                    cosine.acos() // angle in rad
                });
            let signum = tmp.next().map(|v| v.signum()).unwrap();
            let mut diff = false;
            tmp.for_each(|v| {
                diff = v.signum() != signum;
            });
            if diff {
                None
            } else {
                Some(d0)
            }
        });

    if let Some(sdart) = star {
        // if we found a dart from the previous computations, it means the polygon is "fannable"
        // THIS CANNOT BE PARALLELIZED AS IS
        let b0_sdart = cmap.beta::<0>(*sdart);
        cmap.one_unsew(b0_sdart);
        let mut d0 = *sdart;
        for sl in new_darts.chunks_exact(2) {
            let [d1, d2] = sl else { unreachable!() };
            let b1_d0 = cmap.beta::<1>(d0);
            let b1b1_d0 = cmap.beta::<1>(cmap.beta::<1>(d0));
            cmap.one_unsew(b1_d0);
            cmap.two_link(*d1, *d2);
            cmap.one_link(*d2, b1b1_d0);
            cmap.one_sew(b1_d0, *d1);
            cmap.one_sew(*d1, d0);
            d0 = *d2;
        }
        cmap.one_sew(cmap.beta::<1>(cmap.beta::<1>(d0)), d0);
    } else {
        println!("W: "); //TODO: complete
    }
}
