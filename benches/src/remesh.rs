use std::time::Instant;

use honeycomb::{
    core::stm::{atomically, atomically_with_err},
    kernels::remeshing::move_vertex_to_average,
    prelude::{
        CMap2, CoordsFloat, DartIdType, NULL_DART_ID, OrbitPolicy,
        grisubal::{Clip, grisubal},
        triangulation::{TriangulateError, earclip_cell_countercw},
    },
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
    // TODO: replace grisubal with `capture_geometry`
    let mut map: CMap2<T> = grisubal(
        input_map,
        [T::from(args.lx).unwrap(), T::from(args.ly).unwrap()],
        Clip::from(args.clip),
    )
    .unwrap();
    let capture_time = instant.elapsed();

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
        .for_each(|(f, n_d, start)| {
            let new_darts = (start..start + n_d as DartIdType).collect::<Vec<_>>();
            while let Err(e) =
                atomically_with_err(|t| earclip_cell_countercw(t, &map, f, &new_darts))
            {
                match e {
                    TriangulateError::AlreadyTriangulated => break,
                    _ => continue,
                }
            }
        });
    let triangulation_time = instant.elapsed();

    instant = Instant::now();
    // classify_capture(&map)?;
    let classification_time = instant.elapsed();

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

    // TODO: print header

    // -- main remeshing loop
    // a. relax
    // b. cut / collapse
    // c. swap
    // check for ending condition after each relax
    let mut n = 0;
    let mut r;
    loop {
        r = 0;

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
                    atomically(|t| move_vertex_to_average(t, &map, vid, &neighbors));
                });

            r += 1;
            // TODO: make the bound configurable
            if r >= 50 {
                break;
            }
        }
        print!("{}", instant.elapsed().as_millis());

        instant = Instant::now();
        if !args.disable_er {
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
                .filter(|l| (l.to_f64().unwrap() - args.target_length) / args.target_length > 0.2)
                .count();
            // if 95%+ edges are in the target length tolerance range, finish early
            if ((n_e_outside_tol as f32 - n_e as f32).abs() / n_e as f32) < 0.05 {
                print!(" | {}", instant.elapsed().as_millis());
                // TODO: print the rest of the line to have a consistent output
                break;
            }
        }
        print!(" | {}", instant.elapsed().as_millis());

        map.iter_edges().for_each(|e| {
            let n_d = map.add_free_darts(6);
            atomically_with_err(|t| Ok(()));
        });

        todo!();

        n += 1;
        if n >= args.n_rounds.get() {
            break;
        }
    }

    map
}
