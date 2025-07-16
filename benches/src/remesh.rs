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
use rayon::{iter::Either, prelude::*};

use crate::{
    cli::RemeshArgs,
    prof_start, prof_stop,
    utils::{get_num_threads, hash_file},
};

pub fn bench_remesh<T: CoordsFloat>(args: RemeshArgs) -> CMap2<T> {
    let input_map = args.input.to_str().unwrap();
    let target_len = T::from(args.target_length).unwrap();

    let n_threads = if let Ok(val) = get_num_threads() {
        val
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    };

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
        map.par_iter_faces()
            .all(|f| map.orbit(OrbitPolicy::Face, f as DartIdType).count() == 3)
    );
    debug_assert!(map.par_iter_faces().all(|f| {
        map.orbit(OrbitPolicy::FaceLinear, f).count() == 3
            && atomically(|t| check_tri_orientation(t, &map, f as DartIdType))
    }));
    debug_assert!(
        map.par_iter_vertices()
            .all(|v| map.force_read_attribute::<VertexAnchor>(v).is_some())
    );
    debug_assert!(
        map.par_iter_edges()
            .all(|e| map.force_read_attribute::<EdgeAnchor>(e).is_some())
    );
    debug_assert!(
        map.par_iter_faces()
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
        "Round | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {}",
        "# of used darts",     // 15
        "# of unused darts",   // 17
        "Graph compute (s)",   // 17
        "Relax (tot, s)",      // 14
        "Ret cond (s)",        // 12
        "Batch compute (s)",   // 17
        "Dart prealloc (s)",   // 17
        "Cut batch size",      // 14
        "Cut edges (s)",       // 13
        "Collapse batch size", // 19
        "Collapse edges (s)",  // 18
        "Swap edges (s)",      // 14
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
        // not using the map method because it uses a sequential iterator
        let n_unused = (1..map.n_darts() as DartIdType)
            .into_par_iter()
            .filter(|d| map.is_unused(*d))
            .count();
        print!(" | {:>15}", map.n_darts() - n_unused);
        print!(" | {:>17}", n_unused);

        // -- build the vertex graph
        prof_start!("HCBENCH_REMESH_GRAPH");
        instant = Instant::now();
        let nodes: Vec<(_, Vec<_>)> = map
            .par_iter_vertices()
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
            .collect();
        prof_stop!("HCBENCH_REMESH_GRAPH");
        print!(" | {:>17.6e}", instant.elapsed().as_secs_f64());

        // -- relax
        prof_start!("HCBENCH_REMESH_RELAX");
        instant = Instant::now();
        r = 0;
        loop {
            nodes.par_iter().for_each(|(vid, neighbors)| {
                let _ = atomically_with_err(|t| {
                    move_vertex_to_average(t, &map, *vid, &neighbors)?;
                    if !is_orbit_orientation_consistent(t, &map, *vid)? {
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
            map.par_iter_faces()
                .all(|f| { atomically(|t| check_tri_orientation(t, &map, f as DartIdType)) })
        );

        // -- get edges to process
        instant = Instant::now();
        let (long_edges, short_edges): (Vec<_>, Vec<_>) = map
            .par_iter_edges()
            .filter_map(|e| {
                let (l, r) = (e as DartIdType, map.beta::<1>(e as DartIdType));
                let diff =
                    atomically(|t| compute_diff_to_target(t, &map, l, r, args.target_length));
                if diff.abs() > args.target_tolerance {
                    Some((e, diff))
                } else {
                    None
                }
            })
            .partition_map(|(e, diff)| {
                if diff.is_sign_positive() {
                    Either::Left(e)
                } else {
                    Either::Right(e)
                }
            });
        let batch_time = instant.elapsed().as_secs_f64();
        // -- check early return conds if enabled
        if args.enable_er {
            instant = Instant::now();
            let n_e = map.par_iter_edges().count();
            let n_e_outside_tol = long_edges.len() + short_edges.len();
            // if 95%+ edges are in the target length tolerance range, finish early
            if (n_e_outside_tol as f64 / n_e as f64) < args.target_tolerance {
                print!(" | {:>12.6e}", instant.elapsed().as_millis());
                print!(" | {:>13}", "n/a");
                print!(" | {:>12}", "n/a");
                println!(" | {:>8}", "n/a");
                break;
            }
            print!(" | {:>12.6e}", instant.elapsed().as_secs_f64());
        } else {
            print!(" | {:>12}", "n/a");
        }

        // -- preallocate darts for cut phase
        instant = Instant::now();
        let n_e = long_edges.len();
        let n_darts = 6 * n_e;
        let new_darts = if n_unused < n_darts {
            let tmp = map.allocate_unused_darts(n_darts);
            (tmp..tmp + n_darts as DartIdType).collect::<Vec<_>>()
        } else {
            (1..map.n_darts() as DartIdType)
                .into_par_iter()
                .filter(|&d| map.is_unused(d))
                .take_any(n_darts)
                .collect()
        };
        let alloc_time = instant.elapsed().as_secs_f64();
        print!(" | {:>17.6e}", batch_time);
        print!(" | {:>17.6e}", alloc_time);

        // -- cut
        prof_start!("HCBENCH_REMESH_CC");
        print!(" | {:>14}", n_e);
        instant = Instant::now();
        long_edges
            .into_par_iter()
            .zip(new_darts.par_chunks_exact(6))
            .for_each(|(e, sl)| {
                let &[d1, d2, d3, d4, d5, d6] = sl else {
                    unreachable!()
                };

                if map.is_i_free::<2>(e as DartIdType) {
                    while let Err(er) = atomically_with_err(|t| {
                        let nds = [d1, d2, d3];
                        for d in nds {
                            map.claim_dart_transac(t, d)?;
                        }
                        cut_outer_edge(t, &map, e, nds)?;

                        if !is_orbit_orientation_consistent(t, &map, d1)? {
                            abort(SewError::BadGeometry(1, 0, 0))?;
                        }

                        Ok(())
                    }) {
                        match er {
                            // non-recoverable
                            SewError::BadGeometry(1, _, _) => {
                                break;
                            }
                            // inconsistency-related, a retry should work
                            SewError::BadGeometry(_, _, _)
                            | SewError::FailedLink(_)
                            | SewError::FailedAttributeOp(_) => continue,
                        }
                    }
                } else {
                    while let Err(er) = atomically_with_err(|t| {
                        let nds = [d1, d2, d3, d4, d5, d6];
                        for d in nds {
                            map.claim_dart_transac(t, d)?;
                        }

                        cut_inner_edge(t, &map, e, nds)?;

                        if !is_orbit_orientation_consistent(t, &map, d1)? {
                            abort(SewError::BadGeometry(1, 0, 0))?;
                        }

                        Ok(())
                    }) {
                        match er {
                            // non-recoverable
                            SewError::BadGeometry(1, _, _) => {
                                break;
                            }
                            // inconsistency-related, a retry should work
                            SewError::BadGeometry(_, _, _)
                            | SewError::FailedLink(_)
                            | SewError::FailedAttributeOp(_) => continue,
                        }
                    }
                }
            });
        prof_stop!("HCBENCH_REMESH_CUT");
        print!(" | {:>13.6e}", instant.elapsed().as_secs_f64());

        // -- collapse
        prof_start!("HCBENCH_REMESH_COLLAPSE");
        print!(" | {:>19}", short_edges.len());
        instant = Instant::now();
        short_edges.into_par_iter().for_each(|e| {
            while let Err(er) = atomically_with_err(|t| {
                if map.is_unused_transac(t, e as DartIdType)? {
                    // needed as some operations may remove some edges besides the one processed
                    return Ok(());
                }
                let e = map.edge_id_transac(t, e)?;

                let (l, r) = (e as DartIdType, map.beta_transac::<1>(t, e as DartIdType)?);
                let diff = compute_diff_to_target(t, &map, l, r, args.target_length)?;
                if diff.abs() < args.target_tolerance {
                    // edge is within target length tolerance; skip the cut/process phase
                    return Ok(());
                }

                collapse_edge(t, &map, e)?;
                Ok(())
            }) {
                match er {
                    // non-recoverable
                    EdgeCollapseError::FailedCoreOp(SewError::BadGeometry(_, _, _))
                    | EdgeCollapseError::NonCollapsibleEdge(_)
                    | EdgeCollapseError::InvertedOrientation => break,
                    // inconsistency-related, a retry should work
                    EdgeCollapseError::FailedCoreOp(_)
                    | EdgeCollapseError::FailedDartRelease(_)
                    | EdgeCollapseError::BadTopology => continue,
                    // unreachable due to the first `if` of the tx
                    EdgeCollapseError::NullEdge => unreachable!(),
                }
            }
        });
        prof_stop!("HCBENCH_REMESH_COLLAPSE");
        print!(" | {:>18.6e}", instant.elapsed().as_secs_f64());

        // -- swap
        prof_start!("HCBENCH_REMESH_SWAP");
        instant = Instant::now();
        map.par_iter_edges()
            .map(|e| {
                let (l, r) = (e as DartIdType, map.beta::<1>(e as DartIdType));
                let diff =
                    atomically(|t| compute_diff_to_target(t, &map, l, r, args.target_length));
                (e, diff)
            })
            .filter(|(_, diff)| diff.abs() > args.target_tolerance)
            .for_each(|(e, diff)| {
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
                        let (b0l, b0r) =
                            (map.beta_transac::<0>(t, l)?, map.beta_transac::<0>(t, r)?);
                        let new_diff =
                            compute_diff_to_target(t, &map, b0l, b0r, args.target_length)?;

                        // if the swap gets the edge length closer to target value, do it
                        if new_diff.abs() < diff.abs() {
                            swap_edge(t, &map, e)?;
                        }

                        // ensure the swap doesn't invert geometry
                        if !check_tri_orientation(t, &map, l)?
                            || !check_tri_orientation(t, &map, r)?
                        {
                            abort(EdgeSwapError::NotSwappable("swap inverts orientation"))?;
                        }

                        Ok(())
                    }) {
                        match er {
                            EdgeSwapError::NotSwappable(_)
                            | EdgeSwapError::FailedCoreOp(_)
                            | EdgeSwapError::BadTopology => {} // continue
                            EdgeSwapError::NullEdge | EdgeSwapError::IncompleteEdge => {
                                unreachable!()
                            }
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
            });
        prof_stop!("HCBENCH_REMESH_SWAP");
        println!(" | {:>14.6e}", instant.elapsed().as_secs_f64());

        debug_assert!(map.par_iter_faces().all(|f| {
            map.orbit(OrbitPolicy::FaceLinear, f).count() == 3
                && atomically(|t| check_tri_orientation(t, &map, f as DartIdType))
        }));

        n += 1;
        if n >= args.n_rounds.get() {
            break;
        }
    }
    prof_stop!("HCBENCH_REMESH_MAINLOOP");

    debug_assert!(map.par_iter_faces().all(|f| {
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
    let v1 = if let Ok(Some(v)) = map.read_vertex(t, vid1) {
        v
    } else {
        return retry()?;
    };
    let v2 = if let Ok(Some(v)) = map.read_vertex(t, vid2) {
        v
    } else {
        return retry()?;
    };
    let v3 = if let Ok(Some(v)) = map.read_vertex(t, vid3) {
        v
    } else {
        return retry()?;
    };
    Ok(Vertex2::cross_product_from_vertices(&v1, &v2, &v3) > T::zero())
}
