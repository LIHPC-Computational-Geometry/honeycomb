use std::{fs::File, time::Instant};

use honeycomb::{
    core::{
        cmap::CMapError,
        stm::{atomically, StmError},
    },
    prelude::{
        CMap2, CMapBuilder, CoordsFloat, DartIdType, EdgeIdType, Orbit2, OrbitPolicy, Vertex2,
        NULL_DART_ID,
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

macro_rules! process_unsew {
    ($op: expr) => {
        if let Err(e) = $op {
            return match e {
                CMapError::FailedTransaction(e) => Err(e),
                CMapError::FailedAttributeSplit(_) => Err(StmError::Failure),
                CMapError::FailedAttributeMerge(_)
                | CMapError::IncorrectGeometry(_)
                | CMapError::UnknownAttribute(_) => unreachable!(),
            };
        }
    };
}
macro_rules! process_sew {
    ($op: expr) => {
        if let Err(e) = $op {
            return match e {
                CMapError::FailedTransaction(e) => Err(e),
                CMapError::FailedAttributeMerge(_) => Err(StmError::Failure),
                CMapError::FailedAttributeSplit(_)
                | CMapError::IncorrectGeometry(_)
                | CMapError::UnknownAttribute(_) => unreachable!(),
            };
        }
    };
}

fn main() {
    let mut instant = Instant::now();
    // load map from file
    let mut map: CMap2<f64> = CMapBuilder::from(INPUT_MAP).build().unwrap();
    println!("map loaded in {}ms", instant.elapsed().as_millis());

    map.iter_faces().for_each(|f| {
        assert_eq!(
            Orbit2::new(&map, OrbitPolicy::Face, f as DartIdType).count(),
            3,
            "Input mesh isn't a triangle mesh"
        )
    });

    instant = Instant::now();
    // compute first batch
    let mut edges: Vec<EdgeIdType> = fetch_edges_to_process(&map, &TARGET_LENGTH).collect();
    assert_eq!(edges.len(), map.iter_edges().count());
    let mut nd = map.add_free_darts(6 * edges.len()); // 2 for edge split + 2*2 for new edges in neighbor tets
    let mut darts: Vec<DartIdType> = (nd..nd + 6 * edges.len() as DartIdType).collect();
    println!(
        "first batch computed in {}ms",
        instant.elapsed().as_millis()
    );
    let mut step = 0;
    while !edges.is_empty() {
        instant = Instant::now();
        // process edges in parallel with transactions
        let units: Vec<(u32, [u32; 6])> = edges
            .drain(..)
            .zip(darts.chunks(6))
            .map(|(e, sl)| (e, sl.try_into().unwrap()))
            .collect();
        let workloads = if units.len() < 7 {
            units.chunks(units.len()).collect::<Vec<_>>()
        } else {
            units.chunks((units.len() + 1) / 4).collect::<Vec<_>>()
        };
        std::thread::scope(|s| {
            println!("hello");
            for wl in workloads {
                let wl = wl.to_vec();
                s.spawn(|| {
                    println!("batch spawned");
                    wl.into_iter()
                        .for_each(|(e, [nd1, nd2, nd3, nd4, nd5, nd6])| {
                            let on_edge = map.is_i_free::<2>(e as DartIdType);
                            atomically(|trans| {
                                if on_edge {
                                    map.link::<2>(trans, nd1, nd2)?;
                                    map.link::<1>(trans, nd2, nd3)?;
                                    let ld = e as DartIdType;
                                    let (b0ld, b1ld) = (
                                        map.beta_transac::<0>(trans, ld)?,
                                        map.beta_transac::<1>(trans, ld)?,
                                    );
                                    if map.beta_transac::<1>(trans, b1ld)? != b0ld {
                                        println!("wtf");
                                        return Err(StmError::Retry);
                                    }

                                    let (vid1, vid2) = (
                                        map.vertex_id_transac(trans, ld)?,
                                        map.vertex_id_transac(trans, b1ld)?,
                                    );
                                    let new_v = Vertex2::average(
                                        &map.read_vertex(trans, vid1)?.unwrap(),
                                        &map.read_vertex(trans, vid2)?.unwrap(),
                                    );
                                    map.write_vertex(trans, nd1, new_v)?;

                                    process_unsew!(map.unsew::<1>(trans, ld));
                                    process_unsew!(map.unsew::<1>(trans, b1ld));

                                    process_sew!(map.sew::<1>(trans, ld, nd1));
                                    process_sew!(map.sew::<1>(trans, nd1, b0ld));
                                    process_sew!(map.sew::<1>(trans, nd3, b1ld));
                                    process_sew!(map.sew::<1>(trans, b1ld, nd2));

                                    Ok(())
                                } else {
                                    map.link::<2>(trans, nd1, nd2)?;
                                    map.link::<1>(trans, nd2, nd3)?;
                                    map.link::<2>(trans, nd4, nd5)?;
                                    map.link::<1>(trans, nd5, nd6)?;
                                    let (ld, rd) = (
                                        e as DartIdType,
                                        map.beta_transac::<2>(trans, e as DartIdType)?,
                                    );
                                    let (b0ld, b1ld) = (
                                        map.beta_transac::<0>(trans, ld)?,
                                        map.beta_transac::<1>(trans, ld)?,
                                    );
                                    if map.beta_transac::<1>(trans, b1ld)? != b0ld {
                                        println!("wtf");
                                        return Err(StmError::Retry);
                                    }
                                    let (b0rd, b1rd) = (
                                        map.beta_transac::<0>(trans, rd)?,
                                        map.beta_transac::<1>(trans, rd)?,
                                    );
                                    if map.beta_transac::<1>(trans, b1rd)? != b0rd {
                                        println!("wtf");
                                        return Err(StmError::Retry);
                                    }
                                    let (vid1, vid2) = (
                                        map.vertex_id_transac(trans, ld)?,
                                        map.vertex_id_transac(trans, b1ld)?,
                                    );
                                    let new_v = Vertex2::average(
                                        &map.read_vertex(trans, vid1)?.unwrap(),
                                        &map.read_vertex(trans, vid2)?.unwrap(),
                                    );
                                    map.write_vertex(trans, nd1, new_v)?;

                                    process_unsew!(map.unsew::<2>(trans, ld));
                                    process_unsew!(map.unsew::<1>(trans, ld));
                                    process_unsew!(map.unsew::<1>(trans, b1ld));
                                    process_unsew!(map.unsew::<1>(trans, rd));
                                    process_unsew!(map.unsew::<1>(trans, b1rd));

                                    process_sew!(map.sew::<2>(trans, ld, nd6));
                                    process_sew!(map.sew::<2>(trans, rd, nd3));

                                    process_sew!(map.sew::<1>(trans, ld, nd1));
                                    process_sew!(map.sew::<1>(trans, nd1, b0ld));
                                    process_sew!(map.sew::<1>(trans, nd3, b1ld));
                                    process_sew!(map.sew::<1>(trans, b1ld, nd2));

                                    process_sew!(map.sew::<1>(trans, rd, nd4));
                                    process_sew!(map.sew::<1>(trans, nd4, b0rd));
                                    process_sew!(map.sew::<1>(trans, nd6, b1rd));
                                    process_sew!(map.sew::<1>(trans, b1rd, nd5));

                                    Ok(())
                                }
                            });
                        });
                });
            }
        });
        println!("batch processed in {}ms", instant.elapsed().as_millis());

        assert!(
            map.iter_faces()
                .filter(|f| !map.is_free(*f as DartIdType))
                .all(|f| { Orbit2::new(&map, OrbitPolicy::Face, f as DartIdType).count() == 3 }),
            "Input mesh isn't a triangle mesh"
        );

        (1..map.n_darts() as DartIdType).for_each(|d| {
            if map.is_free(d) && !map.is_unused(d) {
                map.remove_free_dart(d);
            }
        });
        let mut f = File::create(format!("step{}.vtk", step)).unwrap();
        map.to_vtk_binary(&mut f);
        step += 1;

        instant = Instant::now();
        // update the edge list
        edges.extend(fetch_edges_to_process(&map, &TARGET_LENGTH));
        // allocate necessary darts
        nd = map.add_free_darts(6 * edges.len());
        darts.par_drain(..); // is there a better way?
        darts.extend(nd..nd + 6 * edges.len() as DartIdType);
        println!("new batch computed in {}ms", instant.elapsed().as_millis());
    }

    // necessary for serialization
    (1..map.n_darts() as DartIdType).for_each(|d| {
        if map.is_free(d) && !map.is_unused(d) {
            map.remove_free_dart(d);
        }
    });

    // checks
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
    assert!(map
        .iter_vertices()
        .all(|v| map.force_read_vertex(v).is_some()));
    assert!(
        map.iter_faces()
            .all(|f| { Orbit2::new(&map, OrbitPolicy::Face, f as DartIdType).count() == 3 }),
        "Input mesh isn't a triangle mesh"
    );

    // serialize
    instant = Instant::now();
    let mut f = File::create("edge_target_size.vtk").unwrap();
    map.to_vtk_binary(&mut f);
    println!("map saved in {}ms", instant.elapsed().as_millis());
}
