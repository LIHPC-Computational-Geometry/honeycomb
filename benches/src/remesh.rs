//! 2D remshing pipeline benchmark
//!
//! This benchmark execute a proxy-application of usual 2D remeshing pipelines. It is split into
//! two parts:
//!
//! - initialization & first capture,
//! - iterative remeshing.
//!
//! Time sampling is done on the second part. Currently only executes in sequential.

use std::time::Instant;

use honeycomb::{
    kernels::{
        grisubal::Clip,
        remeshing::{
            EdgeCollapseError, EdgeSwapError, capture_geometry, classify_capture, collapse_edge,
            cut_inner_edge, cut_outer_edge, move_vertex_to_average, swap_edge,
        },
        triangulation::{TriangulateError, earclip_cell_countercw},
        utils::{EdgeAnchor, FaceAnchor, VertexAnchor, is_orbit_orientation_consistent},
    },
    prelude::{CMap2, CoordsFloat, DartIdType, NULL_DART_ID, OrbitPolicy, SewError, Vertex2},
    stm::{StmClosureResult, Transaction, abort, atomically, atomically_with_err, retry},
};

use crate::{cli::RemeshArgs, prof_start, prof_stop, utils::hash_file};

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
    prof_start!("HCBENCH_REMESH_TRIANGULATION");
    let n_tot = map
        .iter_faces()
        .map(|id| (map.orbit(OrbitPolicy::Face, id as DartIdType).count() - 3) * 2)
        .sum();
    let start = map.allocate_used_darts(n_tot);
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
    let triangulation_time = instant.elapsed();
    prof_stop!("HCBENCH_REMESH_TRIANGULATION");

    // check that the mesh is triangular, consistently oriented and fully classified
    debug_assert!(
        map.iter_faces()
            .all(|f| map.orbit(OrbitPolicy::Face, f as DartIdType).count() == 3)
    );
    debug_assert!(map.iter_faces().all(|f| {
        map.orbit(OrbitPolicy::FaceLinear, f).count() == 3
            && atomically(|t| check_tri_orientation(t, &map, f as DartIdType))
    }));
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

    // TODO: print the whole config / args
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
        "Round | Relax (tot, s) | Ret cond (s) | Batch compute (s) | Cut/collapse (s) | Swap (s)"
    );
    // -- main remeshing loop
    // a. relax
    // b. cut / collapse
    // c. swap
    // check for ending condition after each relax
    prof_start!("HCBENCH_REMESH_MAINLOOP");
    let mut n = 0;
    let mut r;
    loop {
        print!("{:>5}", n);

        // -- relax
        prof_start!("HCBENCH_REMESH_RELAX");
        instant = Instant::now();
        r = 0;
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
            if r >= args.n_relax_rounds.get() {
                break;
            }
        }
        prof_stop!("HCBENCH_REMESH_RELAX");
        print!(" | {:>14.6e}", instant.elapsed().as_secs_f64());

        debug_assert!(
            map.iter_faces()
                .all(|f| { atomically(|t| check_tri_orientation(t, &map, f as DartIdType)) })
        );

        // -- check early return conds if enabled
        if args.enable_er {
            instant = Instant::now();
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
                print!(" | {:>12.6e}", instant.elapsed().as_millis());
                print!(" | {:>17}", "n/a");
                print!(" | {:>16}", "n/a");
                println!(" | {:>8}", "n/a");
                break;
            }
            print!(" | {:>12.6e}", instant.elapsed().as_secs_f64());
        } else {
            print!(" | {:>12}", "n/a");
        }

        // -- get edges to process
        instant = Instant::now();
        let edges_to_process = map.iter_edges().collect::<Vec<_>>();
        print!(" | {:>17.6e}", instant.elapsed().as_secs_f64());

        // -- cut / collapse
        prof_start!("HCBENCH_REMESH_CC");
        instant = Instant::now();
        for e in edges_to_process {
            if map.is_unused(e as DartIdType) {
                // needed as some operations may remove some edges besides the one processed
                continue;
            }

            // filter out
            let (l, r) = (e as DartIdType, map.beta::<1>(e as DartIdType));
            let diff = atomically(|t| compute_diff_to_target(t, &map, l, r, args.target_length));
            if diff.abs() < args.target_tolerance {
                // edge is within target length tolerance; skip the cut/process phase
                continue;
            }
            let e = map.edge_id(e);
            // process
            if diff.is_sign_positive() {
                // edge is longer than target length => cut
                if map.is_i_free::<2>(e as DartIdType) {
                    let nd = map.allocate_used_darts(3);
                    let nds: [DartIdType; 3] = std::array::from_fn(|i| nd + i as DartIdType);
                    while let Err(er) = atomically_with_err(|t| {
                        cut_outer_edge(t, &map, e, nds)?;
                        let new_vid = nds[0];
                        if !is_orbit_orientation_consistent(t, &map, new_vid)? {
                            abort(SewError::BadGeometry(1, nds[0], nds[2]))?;
                        }
                        Ok(())
                    }) {
                        match er {
                            SewError::BadGeometry(1, _, _) => {
                                for d in nds {
                                    map.release_dart(d).expect("E: unreachable");
                                }
                                break;
                            }
                            SewError::BadGeometry(_, _, _)
                            | SewError::FailedLink(_)
                            | SewError::FailedAttributeOp(_) => continue,
                        }
                    }
                } else {
                    let nd = map.allocate_used_darts(6);
                    let nds: [DartIdType; 6] = std::array::from_fn(|i| nd + i as DartIdType);
                    while let Err(er) = atomically_with_err(|t| {
                        cut_inner_edge(t, &map, e, nds)?;
                        let new_vid = nds[0];
                        if !is_orbit_orientation_consistent(t, &map, new_vid)? {
                            abort(SewError::BadGeometry(1, nds[0], nds[3]))?;
                        }
                        Ok(())
                    }) {
                        match er {
                            SewError::BadGeometry(1, _, _) => {
                                for d in nds {
                                    map.release_dart(d).expect("E: unreachable");
                                }
                                break;
                            }
                            SewError::BadGeometry(_, _, _)
                            | SewError::FailedLink(_)
                            | SewError::FailedAttributeOp(_) => continue,
                        }
                    }
                }
            } else {
                // edge is shorter than target length => collapse
                while let Err(er) = atomically_with_err(|t| collapse_edge(t, &map, e)) {
                    match er {
                        EdgeCollapseError::FailedCoreOp(SewError::BadGeometry(_, _, _))
                        | EdgeCollapseError::NonCollapsibleEdge(_)
                        | EdgeCollapseError::InvertedOrientation => break,
                        EdgeCollapseError::FailedCoreOp(_)
                        | EdgeCollapseError::FailedDartRelease(_)
                        | EdgeCollapseError::BadTopology => continue,
                        EdgeCollapseError::NullEdge => unreachable!(),
                    }
                }
            }
        }
        prof_stop!("HCBENCH_REMESH_CC");
        print!(" | {:>16.6e}", instant.elapsed().as_secs_f64());

        // -- swap
        prof_start!("HCBENCH_REMESH_SWAP");
        instant = Instant::now();
        for (e, diff) in map
            .iter_edges()
            .map(|e| {
                let (l, r) = (e as DartIdType, map.beta::<1>(e as DartIdType));
                let diff =
                    atomically(|t| compute_diff_to_target(t, &map, l, r, args.target_length));
                (e, diff)
            })
            .filter(|(_, diff)| diff.abs() > args.target_tolerance)
        {
            let (l, r) = (e as DartIdType, map.beta::<2>(e as DartIdType));
            if r != NULL_DART_ID {
                debug_assert!(
                    map.force_read_attribute::<FaceAnchor>(map.face_id(l))
                        .is_some()
                );
                debug_assert!(
                    map.force_read_attribute::<FaceAnchor>(map.face_id(r))
                        .is_some()
                );
                if let Err(er) = atomically_with_err(|t| {
                    let (b0l, b0r) = (map.beta_transac::<0>(t, l)?, map.beta_transac::<0>(t, r)?);
                    let new_diff = compute_diff_to_target(t, &map, b0l, b0r, args.target_length)?;

                    // if the swap gets the edge length closer to target value, do it
                    if new_diff.abs() < diff.abs() {
                        swap_edge(t, &map, e)?;
                    }

                    // ensure the swap doesn't invert geometry
                    if !check_tri_orientation(t, &map, l)? || !check_tri_orientation(t, &map, r)? {
                        abort(EdgeSwapError::NotSwappable("swap inverts orientation"))?;
                    }

                    Ok(())
                }) {
                    match er {
                        EdgeSwapError::NotSwappable(_) => {} // continue
                        EdgeSwapError::NullEdge
                        | EdgeSwapError::IncompleteEdge
                        | EdgeSwapError::FailedCoreOp(_)
                        | EdgeSwapError::BadTopology => unreachable!(),
                    }
                }

                debug_assert!(
                    map.force_read_attribute::<FaceAnchor>(map.face_id(l))
                        .is_some()
                );
                debug_assert!(
                    map.force_read_attribute::<FaceAnchor>(map.face_id(r))
                        .is_some()
                );
            }
        }
        prof_stop!("HCBENCH_REMESH_SWAP");
        println!(" | {:>8.6e}", instant.elapsed().as_secs_f64());

        debug_assert!(map.iter_faces().all(|f| {
            map.orbit(OrbitPolicy::FaceLinear, f).count() == 3
                && atomically(|t| check_tri_orientation(t, &map, f as DartIdType))
        }));

        n += 1;
        if n >= args.n_rounds.get() {
            break;
        }
    }
    prof_stop!("HCBENCH_REMESH_MAINLOOP");

    debug_assert!(map.iter_faces().all(|f| {
        map.orbit(OrbitPolicy::FaceLinear, f).count() == 3
            && atomically(|t| check_tri_orientation(t, &map, f as DartIdType))
    }));

    map
}

#[inline]
fn compute_diff_to_target<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    l: DartIdType,
    r: DartIdType,
    target: f64,
) -> StmClosureResult<f64> {
    let (vid1, vid2) = (map.vertex_id_transac(t, l)?, map.vertex_id_transac(t, r)?);
    let (v1, v2) =
        if let (Some(v1), Some(v2)) = (map.read_vertex(t, vid1)?, map.read_vertex(t, vid2)?) {
            (v1, v2)
        } else {
            retry()?
        };
    Ok(((v2 - v1).norm().to_f64().unwrap() - target) / target)
}

#[inline]
fn check_tri_orientation<T: CoordsFloat>(
    t: &mut Transaction,
    map: &CMap2<T>,
    d: DartIdType,
) -> StmClosureResult<bool> {
    let vid1 = map.vertex_id_transac(t, d)?;
    let b1 = map.beta_transac::<1>(t, d)?;
    let vid2 = map.vertex_id_transac(t, b1)?;
    let b1b1 = map.beta_transac::<1>(t, b1)?;
    let vid3 = map.vertex_id_transac(t, b1b1)?;
    let v1 = map.read_vertex(t, vid1)?.unwrap();
    let v2 = map.read_vertex(t, vid2)?.unwrap();
    let v3 = map.read_vertex(t, vid3)?.unwrap();
    Ok(Vertex2::cross_product_from_vertices(&v1, &v2, &v3) > T::zero())
}
