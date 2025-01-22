use std::time::Instant;

use honeycomb::{
    core::stm::atomically,
    prelude::{
        splits::{split_edge_transac, SplitEdgeError},
        CMap2, CMapBuilder, CoordsFloat, DartIdType, EdgeIdType,
    },
};

use rayon::prelude::*;

const INPUT_MAP: &str = "grid_split.vtk";
const TARGET_LENGTH: f64 = 0.1;

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
        let (v1, v2) = (
            map.force_read_vertex(vid1).unwrap(),
            map.force_read_vertex(vid2).unwrap(),
        );

        (v2 - v1).norm() > *length
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
        edges
            .par_drain(..)
            .zip(darts.par_chunks(6))
            .for_each(|(e, sl)| {
                // we can read invariants outside of the transaction
                let &[nd1, nd2, nd3, nd4, nd5, nd6] = sl else {
                    unreachable!()
                };
                let (ld, rd) = (e as DartIdType, map.beta::<2>(e as DartIdType));

                atomically(|trans| {
                    // adding a b0 read helps with conflict detection
                    let (_b0ld, b1ld) = (
                        map.beta_transac::<0>(trans, ld)?,
                        map.beta_transac::<1>(trans, ld)?,
                    );
                    let (_b0rd, b1rd) = (
                        map.beta_transac::<0>(trans, rd)?,
                        map.beta_transac::<1>(trans, rd)?,
                    );

                    if let Err(e) = split_edge_transac(&map, trans, e, (nd1, nd2), None) {
                        match e {
                            SplitEdgeError::FailedTransaction(stmerr) => return Err(stmerr),
                            SplitEdgeError::UndefinedEdge => unreachable!("unreachable due to STM"),
                            SplitEdgeError::VertexBound
                            | SplitEdgeError::InvalidDarts(_)
                            | SplitEdgeError::WrongAmountDarts(_, _) => unreachable!(),
                        }
                    };

                    // left side tet
                    map.unlink::<1>(trans, ld)?;
                    map.unlink::<1>(trans, b1ld)?;
                    map.link::<2>(trans, nd3, nd4)?;
                    map.link::<1>(trans, ld, nd4)?;
                    map.link::<1>(trans, nd4, nd2)?;
                    // right side tet
                    map.unlink::<1>(trans, rd)?;
                    map.unlink::<1>(trans, b1rd)?;
                    map.link::<2>(trans, nd5, nd6)?;
                    map.link::<1>(trans, rd, nd6)?;
                    map.link::<1>(trans, nd5, nd1)?;

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
}
