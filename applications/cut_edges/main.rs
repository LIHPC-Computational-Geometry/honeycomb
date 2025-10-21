mod cli;
mod dispatch;

use std::{io::Write, path::PathBuf, time::Instant};

use clap::Parser;
use honeycomb::prelude::{CMap2, CMapBuilder, CoordsFloat, DartIdType, EdgeIdType};
#[cfg(feature = "render")]
use honeycomb::render::render_2d_map;
use rayon::prelude::*;

use applications::{
    Backend, FileFormat, bind_rayon_threads, get_num_threads, hash_file, prof_start, prof_stop,
};

#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(cli.input, cli.target_length, cli.backend, cli.save_as);
    } else {
        run_bench::<f64>(cli.input, cli.target_length, cli.backend, cli.save_as);
    }
}

// const MAX_RETRY: u8 = 10;

pub fn run_bench<T: CoordsFloat>(
    input: PathBuf,
    target_length: f64,
    backend: Backend,
    save: Option<FileFormat>,
) {
    let input_map = input.to_str().unwrap();
    let target_len = T::from(target_length).unwrap();
    let n_threads = if let Ok(val) = get_num_threads() {
        val
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    };

    // load map from file
    let instant = Instant::now();
    let input_hash = hash_file(input_map).expect("E: could not compute input hash"); // file id for posterity

    let mut map: CMap2<T> = if input_map.ends_with(".cmap") {
        CMapBuilder::<2>::from_cmap_file(input_map).build().unwrap()
    } else if input_map.ends_with(".vtk") {
        CMapBuilder::<2>::from_vtk_file(input_map).build().unwrap()
    } else {
        panic!(
            "E: Unknown file format; only .cmap or .vtk files are supported for map initialization"
        );
    };
    #[cfg(debug_assertions)] // check input
    {
        use honeycomb::prelude::OrbitPolicy;
        assert!(
            map.iter_faces()
                .all(|f| { map.orbit(OrbitPolicy::Face, f as DartIdType).count() == 3 }),
            "Input mesh isn't a triangle mesh"
        );
    }
    println!("| cut-edges benchmark");
    println!("|-> input      : {input_map} (hash: {input_hash:#0x})",);
    println!("|-> backend    : {:?} with {n_threads} thread(s)", backend);
    println!("|-> target size: {target_len:?}");
    println!("|-> init time  : {}ms", instant.elapsed().as_millis());

    benchmark(&mut map, target_len, backend, n_threads);

    match save {
        Some(FileFormat::Cmap) => {
            // FIXME: update serialize sig
            let mut out = String::new();
            let mut file = std::fs::File::create("out.cmap").unwrap();
            map.serialize(&mut out);
            file.write_all(out.as_bytes()).unwrap();
        }
        Some(FileFormat::Vtk) => {
            let mut file = std::fs::File::create("out.vtk").unwrap();
            map.to_vtk_binary(&mut file);
        }
        None => {}
    }

    #[cfg(feature = "render")]
    {
        render_2d_map(&map);
    }
}

fn benchmark<T: CoordsFloat>(
    map: &mut CMap2<T>,
    target_length: T,
    backend: Backend,
    n_threads: usize,
) {
    println!(
        " Step | n_edge_total | n_edge_to_process | t_compute_batch(s) | t_process_batch(s) | n_tx_retry"
    );

    let mut step = 0;
    print!(" {step:>4} "); // Step
    prof_start!("HCBENCH_CUTS");

    // compute first batch
    let n_e_total = map.par_iter_edges().count();
    prof_start!("HCBENCH_CUTS_COMPUTE");
    let mut instant = Instant::now();
    let mut edges: Vec<EdgeIdType> = map
        .par_iter_edges()
        .filter(|&e| {
            let (vid1, vid2) = (
                map.vertex_id(e as DartIdType),
                map.vertex_id(map.beta::<1>(e as DartIdType)),
            );
            match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
                (Some(v1), Some(v2)) => (v2 - v1).norm() > target_length,
                (_, _) => false,
            }
        })
        .collect();
    let n_e = edges.len();
    print!("| {n_e_total:>12} ");
    print!("| {n_e:>17} ");
    let mut nd = map.allocate_used_darts(6 * n_e); // 6 darts needed per cut
    let mut darts: Vec<DartIdType> = (nd..nd + 6 * n_e as DartIdType).into_par_iter().collect();
    prof_stop!("HCBENCH_CUTS_COMPUTE");
    print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch

    // while there are edges to cut
    while !edges.is_empty() {
        // process batch
        prof_start!("HCBENCH_CUTS_PROCESS");
        instant = Instant::now();
        let n_retry = match backend {
            Backend::RayonIter => dispatch::dispatch_rayon(&map, &mut edges, &darts),
            Backend::RayonChunks => {
                dispatch::dispatch_rayon_chunks(&map, &mut edges, &darts, n_threads)
            }
            Backend::StdThreads => {
                dispatch::dispatch_std_threads(&map, &mut edges, &darts, n_threads)
            }
        };
        prof_stop!("HCBENCH_CUTS_PROCESS");
        print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_process_batch
        println!("| {n_retry:>15}",); // n_tx_retry

        (1..map.n_darts() as DartIdType).for_each(|d| {
            if map.is_free(d) && !map.is_unused(d) {
                map.release_dart(d).expect("E: unreachable");
            }
        });

        // compute the new batch
        step += 1;
        let n_e_total = map.par_iter_edges().count();
        print!(" {step:>4} "); // Step
        prof_start!("HCBENCH_CUTS_COMPUTE");
        instant = Instant::now();
        edges.par_extend(map.par_iter_edges().filter(|&e| {
            let (vid1, vid2) = (
                map.vertex_id(e as DartIdType),
                map.vertex_id(map.beta::<1>(e as DartIdType)),
            );
            match (map.force_read_vertex(vid1), map.force_read_vertex(vid2)) {
                (Some(v1), Some(v2)) => (v2 - v1).norm() > target_length,
                (_, _) => false,
            }
        }));
        print!("| {n_e_total:>12} ");
        let n_e = edges.len();
        print!("| {n_e:>17} ");
        nd = map.allocate_used_darts(6 * n_e);
        darts.par_drain(..); // is there a better way?
        darts.extend(nd..nd + 6 * n_e as DartIdType);
        prof_stop!("HCBENCH_CUTS_COMPUTE");
        if n_e != 0 {
            print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch
        } else {
            print!("| {:>18.6e} ", instant.elapsed().as_secs_f64()); // t_compute_batch
            print!("| {:>18.6e} ", 0.0); // t_process_batch
            println!("| {:>15}", 0); // n_tx_retry
        }
    }
    prof_stop!("HCBENCH_CUTS");
}
