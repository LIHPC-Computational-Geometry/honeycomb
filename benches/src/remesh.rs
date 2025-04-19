use std::time::Instant;

use honeycomb::{
    core::stm::{atomically, atomically_with_err},
    kernels::{
        remeshing::move_vertex_to_average,
        utils::{EdgeAnchor, FaceAnchor, VertexAnchor, is_orbit_orientation_consistent},
    },
    prelude::{
        CMap2, CoordsFloat, DartIdType, NULL_DART_ID, OrbitPolicy, Vertex2,
        grisubal::Clip,
        remeshing::{
            EdgeCollapseError, EdgeSwapError, capture_geometry, classify_capture, collapse_edge,
            cut_inner_edge, cut_outer_edge, swap_edge,
        },
        triangulation::{TriangulateError, earclip_cell_countercw},
    },
    stm::{abort, retry},
};

use crate::{cli::RemeshArgs, utils::hash_file};

pub fn bench_remesh<T: CoordsFloat>(args: RemeshArgs) -> CMap2<T> {
    let input_map = args.input.to_str().unwrap();
    let target_len = T::from(args.target_length).unwrap();

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    // load map from file
    let input_hash = hash_file(input_map).expect("E: could not compute input hash"); // file id for posterity

    // -- capture via grid overlap
    let mut instant = Instant::now();
    let mut map: CMap2<T> = capture_geometry(
        input_map,
        [T::from(args.lx).unwrap(), T::from(args.ly).unwrap()],
        Clip::from(args.clip),
    )
    .unwrap();
    let capture_time = instant.elapsed();

    // -- classification
    instant = Instant::now();
    classify_capture(&map).unwrap();
    let classification_time = instant.elapsed();

    // -- triangulation
    instant = Instant::now();
    let n_tot = map
        .iter_faces()
        .map(|id| (map.orbit(OrbitPolicy::Face, id as DartIdType).count() - 3) * 2)
        .sum();
    let start = map.add_free_darts(n_tot);
    // use a prefix sum starting from the newly allocated darts to associate free darts to each face
    map.iter_faces()
        .scan(start, |state, f| {
            // compute the number of dart needed to triangulate this face
            let n_d = (map.orbit(OrbitPolicy::Face, f as DartIdType).count() - 3) * 2;
            *state += n_d as DartIdType;
            Some((f, n_d, *state - n_d as DartIdType))
        })
        .filter(|(_, n_d, _)| *n_d != 0)
        .for_each(|(f, n_d, start)| {
            let new_darts = (start..start + n_d as DartIdType).collect::<Vec<_>>();
            let anchor = map.force_remove_attribute::<FaceAnchor>(f);
            // make sure new edges are anchored
            if let Some(a) = anchor {
                atomically(|t| {
                    for &d in &new_darts {
                        map.write_attribute(t, d, EdgeAnchor::from(a))?;
                    }
                    Ok(())
                });
            }
            while let Err(e) =
                atomically_with_err(|t| earclip_cell_countercw(t, &map, f, &new_darts))
            {
                match e {
                    TriangulateError::UndefinedFace(_) | TriangulateError::OpFailed(_) => continue,
                    TriangulateError::NoEar => panic!("E: cannot triangulate the geometry capture"),
                    TriangulateError::AlreadyTriangulated
                    | TriangulateError::NonFannable
                    | TriangulateError::NotEnoughDarts(_)
                    | TriangulateError::TooManyDarts(_) => {
                        unreachable!()
                    }
                }
            }
            // make sure new faces are anchored
            if let Some(a) = anchor {
                atomically(|t| {
                    for &d in &new_darts {
                        let fid = map.face_id_transac(t, d)?;
                        map.write_attribute(t, fid, a)?;
                    }
                    Ok(())
                });
            }
        });
    // these assertions check that the mesh is triangular and fully classified
    debug_assert!(
        map.iter_faces()
            .all(|f| map.orbit(OrbitPolicy::Face, f as DartIdType).count() == 3)
    );
    debug_assert!(
        map.iter_vertices()
            .all(|v| map.force_read_attribute::<VertexAnchor>(v).is_some())
    );
    debug_assert!(
        map.iter_edges()
            .all(|e| map.force_read_attribute::<EdgeAnchor>(e).is_some())
    );
    debug_assert!(
        map.iter_faces()
            .all(|f| map.force_read_attribute::<FaceAnchor>(f).is_some())
    );
    let triangulation_time = instant.elapsed();

    // TODO: print the whole config
    println!("| remesh benchmark");
    println!("|-> input      : {input_map} (hash: {input_hash:#0x})");
    println!(
        "|-> backend    : {:?} with {n_threads} thread(s)",
        args.backend
    );
    println!("|-> target size: {target_len:?}");
    println!("|-> capture time  : {}ms", capture_time.as_millis());
    println!(
        "|-> triangulation time  : {}ms",
        triangulation_time.as_millis()
    );
    println!(
        "|-> classification time  : {}ms",
        classification_time.as_millis()
    );

    println!(
        "Round | Relax (avg, s) | Ret cond (s) | Batch compute (s) | Cut/collapse (s) | Swap (s)"
    );

    // -- main remeshing loop
    // a. relax
    // b. cut / collapse
    // c. swap
    // check for ending condition after each relax
    let mut n = 0;
    let mut r;
    loop {
        print!("{:>5}", n);
        r = 0;

        // -- relax
        instant = Instant::now();
        loop {
            map.iter_vertices()
                .filter_map(|v| {
                    let mut neigh = Vec::with_capacity(10);
                    for d in map.orbit(OrbitPolicy::Vertex, v as DartIdType) {
                        let b2d = map.beta::<2>(d);
                        if b2d == NULL_DART_ID {
                            return None; // filter out vertices on the boundary
                        } else {
                            neigh.push(map.vertex_id(b2d));
                        }
                    }
                    Some((v, neigh))
                })
                .for_each(|(vid, neighbors)| {
                    let _ = atomically_with_err(|t| {
                        move_vertex_to_average(t, &map, vid, &neighbors)?;
                        if !is_orbit_orientation_consistent(t, &map, vid)? {
                            abort("E: resulting geometry is inverted")?;
                        }
                        Ok(())
                    });
                });

            r += 1;
            // TODO: make the bound configurable
            if r >= 50 {
                break;
            }
        }
        print!(" | {:>14.6e}", instant.elapsed().as_secs_f64() / 50.0);

        assert!(
            map.iter_vertices()
                .all(|v| atomically(|t| is_orbit_orientation_consistent(t, &map, v)))
        );

        instant = Instant::now();
        if args.disable_er {
            print!(" | {:>12}", "n/a");
        } else {
            let n_e = map.iter_edges().count();
            let n_e_outside_tol = map
                .iter_edges()
                .map(|e| {
                    let (v1, v2) = (
                        map.force_read_vertex(map.vertex_id(e as DartIdType))
                            .unwrap(),
                        map.force_read_vertex(map.vertex_id(map.beta::<1>(e as DartIdType)))
                            .unwrap(),
                    );
                    (v2 - v1).norm()
                })
                .filter(|l| {
                    (l.to_f64().unwrap() - args.target_length).abs() / args.target_length
                        > args.target_tolerance
                })
                .count();
            // if 95%+ edges are in the target length tolerance range, finish early
            if ((n_e_outside_tol as f32 - n_e as f32).abs() / n_e as f32) < 0.05 {
                print!(" | {}", instant.elapsed().as_millis());
                // TODO: print the rest of the line to have a consistent output
                break;
            }
            print!(" | {:>12.6e}", instant.elapsed().as_secs_f64());
        }

        instant = Instant::now();
        let edges_to_process = map
            .iter_edges()
            .map(|e| {
                let (v1, v2) = (
                    map.force_read_vertex(map.vertex_id(e as DartIdType))
                        .unwrap(),
                    map.force_read_vertex(map.vertex_id(map.beta::<1>(e as DartIdType)))
                        .unwrap(),
                );
                let norm = (v2 - v1).norm();
                (
                    e,
                    (norm.to_f64().unwrap() - args.target_length) / args.target_length,
                )
            })
            .filter(|(_, diff)| diff.abs() > args.target_tolerance)
            .collect::<Vec<_>>();
        print!(" | {:>17.6e}", instant.elapsed().as_secs_f64());

        // -- cut / collapse
        instant = Instant::now();
        for (e, diff) in edges_to_process {
            if !map.is_unused(e as DartIdType) {
                if diff.is_sign_positive() {
                    // edge is 20+% longer than target length => cut
                    if map.is_i_free::<2>(e as DartIdType) {
                        assert!(map.force_read_attribute::<EdgeAnchor>(e).is_some());
                        let new_v = map.vertex_id(map.beta::<1>(e as DartIdType));
                        assert!(map.orbit(OrbitPolicy::Vertex, new_v).all(|d| {
                            map.force_read_attribute::<EdgeAnchor>(map.edge_id(d))
                                .is_some()
                        }));
                        let nd = map.add_free_darts(3);
                        let nds: [DartIdType; 3] = std::array::from_fn(|i| nd + i as DartIdType);
                        while let Err(e) = atomically_with_err(|t| cut_outer_edge(t, &map, e, nds))
                        {
                            eprintln!("dumbass");
                            eprintln!("{e}");
                        }
                        assert!(map.force_read_attribute::<EdgeAnchor>(e).is_some());
                        let new_v = map.vertex_id(map.beta::<1>(e as DartIdType));
                        assert!(map.orbit(OrbitPolicy::Vertex, new_v).all(|d| {
                            map.force_read_attribute::<EdgeAnchor>(map.edge_id(d))
                                .is_some()
                        }));
                    } else {
                        assert!(map.force_read_attribute::<EdgeAnchor>(e).is_some());
                        let new_v = map.vertex_id(map.beta::<1>(e as DartIdType));
                        assert!(map.orbit(OrbitPolicy::Vertex, new_v).all(|d| {
                            map.force_read_attribute::<EdgeAnchor>(map.edge_id(d))
                                .is_some()
                        }));
                        let nd = map.add_free_darts(6);
                        let nds: [DartIdType; 6] = std::array::from_fn(|i| nd + i as DartIdType);
                        while let Err(e) = atomically_with_err(|t| cut_inner_edge(t, &map, e, nds))
                        {
                            eprintln!("{e}");
                        }

                        assert!(map.force_read_attribute::<EdgeAnchor>(e).is_some());
                        let new_v = map.vertex_id(map.beta::<1>(e as DartIdType));
                        assert!(map.orbit(OrbitPolicy::Vertex, new_v).all(|d| {
                            map.force_read_attribute::<EdgeAnchor>(map.edge_id(d))
                                .is_some()
                        }));
                    }
                } else {
                    // edge is 20+% shorter than target length => collapse

                    match atomically_with_err(|t| collapse_edge(t, &map, e)) {
                        Ok(new_v) => {
                            if new_v != 0 {
                                assert!(map.orbit(OrbitPolicy::Vertex, new_v).all(|d| {
                                    map.force_read_attribute::<EdgeAnchor>(map.edge_id(d))
                                        .is_some()
                                }));
                            }
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            match e {
                                EdgeCollapseError::FailedCoreOp(_)
                                | EdgeCollapseError::BadTopology => {
                                    continue;
                                }
                                EdgeCollapseError::NonCollapsibleEdge(_)
                                | EdgeCollapseError::InvertedOrientation => break,
                                EdgeCollapseError::NullEdge => unreachable!(),
                            }
                        }
                    }
                }
            }
        }
        print!(" | {:>16.6e}", instant.elapsed().as_secs_f64());

        assert!(
            map.iter_vertices()
                .all(|v| atomically(|t| is_orbit_orientation_consistent(t, &map, v)))
        );

        // -- swap
        instant = Instant::now();
        for (e, diff) in map
            .iter_edges()
            .map(|e| {
                let (v1, v2) = (
                    map.force_read_vertex(map.vertex_id(e as DartIdType))
                        .unwrap(),
                    map.force_read_vertex(map.vertex_id(map.beta::<1>(e as DartIdType)))
                        .unwrap(),
                );
                let norm = (v2 - v1).norm();
                (
                    e,
                    (norm.to_f64().unwrap() - args.target_length) / args.target_length,
                )
            })
            .filter(|(_, diff)| diff.abs() > args.target_tolerance)
        {
            let (l, r) = (e as DartIdType, map.beta::<2>(e as DartIdType));
            if r != NULL_DART_ID {
                while let Err(e) = atomically_with_err(|t| {
                    let (vid1, vid2) = (map.vertex_id_transac(t, l)?, map.vertex_id_transac(t, r)?);
                    let (v1, v2) = if let (Some(v1), Some(v2)) =
                        (map.read_vertex(t, vid1)?, map.read_vertex(t, vid2)?)
                    {
                        (v1, v2)
                    } else {
                        retry()?
                    };
                    let norm = (v2 - v1).norm();
                    let new_diff =
                        (norm.to_f64().unwrap() - args.target_length) / args.target_length;

                    // if the swap gets the edge length closer to target value, do it
                    if new_diff.abs() < diff.abs() {
                        swap_edge(t, &map, e)?;
                    }

                    if !is_orbit_orientation_consistent(t, &map, vid1)? {
                        abort(EdgeSwapError::IncompleteEdge)?; // hacky for now
                    }

                    Ok(())
                }) {
                    match e {
                        EdgeSwapError::FailedCoreOp(_) | EdgeSwapError::BadTopology => continue,
                        EdgeSwapError::IncompleteEdge | EdgeSwapError::NullEdge => unreachable!(),
                    }
                }
            }
        }
        println!(" | {:>8.6e}", instant.elapsed().as_secs_f64());

        assert!(
            map.iter_vertices()
                .all(|v| atomically(|t| is_orbit_orientation_consistent(t, &map, v)))
        );

        n += 1;
        if n >= args.n_rounds.get() {
            break;
        }
    }

    map.iter_faces().for_each(|f| {
        assert!(map.orbit(OrbitPolicy::FaceLinear, f).count() == 3);
        let vid1 = map.vertex_id(f);
        let vid2 = map.vertex_id(map.beta::<1>(f));
        let vid3 = map.vertex_id(map.beta::<1>(map.beta::<1>(f)));
        let v1 = map.force_read_vertex(vid1).unwrap();
        let v2 = map.force_read_vertex(vid2).unwrap();
        let v3 = map.force_read_vertex(vid3).unwrap();
        assert!(Vertex2::cross_product_from_vertices(&v1, &v2, &v3) < T::zero());
    });

    map
}
