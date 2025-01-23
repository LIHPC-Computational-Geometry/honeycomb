use std::{fs::File, time::Instant};

use honeycomb::{
    core::stm::atomically,
    prelude::{
        splits::{split_edge_transac, SplitEdgeError},
        CMap2, CMapBuilder, CoordsFloat, DartIdType, EdgeIdType, NULL_DART_ID,
    },
};

use rayon::prelude::*;

const INPUT_MAP: &str = "grid_split.vtk";
const TARGET_LENGTH: f64 = 0.4;

fn fetch_edges_to_process<'a, 'b, T: CoordsFloat>(
    map: &'a CMap2<T>,
    length: &'b T,
) -> impl Iterator<Item = EdgeIdType> + 'a
where
    'b: 'a,
{
    map.iter_edges().filter(|&e| {
        let (vid1, vid2) = (
            map.vertex_id(e as DartIdType),
            map.vertex_id(map.beta::<1>(e as DartIdType)),
        );
        match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
            (Some(v1), Some(v2)) => (v2 - v1).norm() > *length,
            (_, _) => false,
        }
    })
}

fn main() {
    let mut instant = Instant::now();
    // load map from file
    let mut map: CMap2<f64> = CMapBuilder::from(INPUT_MAP).build().unwrap();
    println!("map loaded in {}ms", instant.elapsed().as_millis());

    instant = Instant::now();
    // compute first batch
    let mut edges: Vec<EdgeIdType> = fetch_edges_to_process(&map, &TARGET_LENGTH).collect();
    let mut nd = map.add_free_darts(6 * edges.len()); // 2 for edge split + 2*2 for new edges in neighbor tets
    let mut darts: Vec<DartIdType> = (nd..nd + 6 * edges.len() as DartIdType).collect();
    println!(
        "first batch computed in {}ms",
        instant.elapsed().as_millis()
    );

    while !edges.is_empty() {
        instant = Instant::now();
        // process edges in parallel with transactions
        edges.drain(..).zip(darts.chunks(6)).for_each(|(e, sl)| {
            // we can read invariants outside of the transaction
            let &[nd1, nd2, nd3, nd4, nd5, nd6] = sl else {
                unreachable!()
            };
            let (ld, rd) = (e as DartIdType, map.beta::<2>(e as DartIdType));

            atomically(|trans| {
                let (b0ld, b1ld) = (
                    map.beta_transac::<0>(trans, ld)?,
                    map.beta_transac::<1>(trans, ld)?,
                );
                let (b0rd, b1rd) = if rd == NULL_DART_ID {
                    (NULL_DART_ID, NULL_DART_ID)
                } else {
                    (
                        map.beta_transac::<0>(trans, rd)?,
                        map.beta_transac::<1>(trans, rd)?,
                    )
                };

                if let Err(e) = split_edge_transac(&map, trans, e, (nd1, nd2), None) {
                    match e {
                        SplitEdgeError::FailedTransaction(stmerr) => return Err(stmerr),
                        SplitEdgeError::UndefinedEdge => unreachable!("unreachable due to STM"),
                        SplitEdgeError::VertexBound
                        | SplitEdgeError::InvalidDarts(_)
                        | SplitEdgeError::WrongAmountDarts(_, _) => unreachable!(),
                    }
                };

                // left side
                // unlink original tet
                map.unlink::<1>(trans, ld)?;
                map.unlink::<1>(trans, b1ld)?;
                // build the new edge
                map.link::<2>(trans, nd3, nd4)?;
                // build 1st new tet
                map.link::<1>(trans, ld, nd4)?;
                map.link::<1>(trans, nd4, b0ld)?;
                // build 2nd new tet
                map.link::<1>(trans, b1ld, nd3)?;
                map.link::<1>(trans, nd3, nd1)?;

                // right side, if there was one
                if rd != NULL_DART_ID {
                    // unlink original tet
                    map.unlink::<1>(trans, rd)?;
                    map.unlink::<1>(trans, b1rd)?;
                    // build the new edge
                    map.link::<2>(trans, nd5, nd6)?;
                    // build 1st new tet
                    map.link::<1>(trans, rd, nd6)?;
                    map.link::<1>(trans, nd6, b0rd)?;
                    // build 2nd new tet
                    map.link::<1>(trans, b1rd, nd5)?;
                    map.link::<1>(trans, nd5, nd2)?;
                }

                Ok(())
            });
        });
        println!("batch processed in {}ms", instant.elapsed().as_millis());

        instant = Instant::now();
        // update the edge list
        edges.extend(fetch_edges_to_process(&map, &TARGET_LENGTH));
        // allocate necessary darts
        nd = map.add_free_darts(6 * edges.len());
        darts.par_drain(..); // is there a better way?
        darts.extend(nd..nd + 6 * edges.len() as DartIdType);
        println!("new batch computed in {}ms", instant.elapsed().as_millis());
    }

    assert!(map
        .iter_edges()
        .filter_map(|e| {
            let (vid1, vid2) = (
                map.vertex_id(e as DartIdType),
                map.vertex_id(map.beta::<1>(e as DartIdType)),
            );
            match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
                (Some(v1), Some(v2)) => Some((v2 - v1).norm()),
                (_, _) => None,
            }
        })
        .all(|norm| norm <= TARGET_LENGTH));

    (1..map.n_darts() as DartIdType).for_each(|d| {
        if map.is_free(d) {
            map.remove_free_dart(d);
        }
    });

    instant = Instant::now();
    // (c) save the map
    let mut f = File::create("edge_target_size.vtk").unwrap();
    map.to_vtk_binary(&mut f);
    println!("map saved in {}ms", instant.elapsed().as_millis());
}
