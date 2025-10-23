mod cli;
mod internals;

use std::path::PathBuf;

use clap::Parser;
use honeycomb::prelude::{CoordsFloat, DartIdType};
use rayon::prelude::*;

use applications::{
    Backend, FileFormat, bind_rayon_threads, finalize_2d, get_num_threads, init_2d_map_from_file,
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
    let (mut map, input_hash, init_time) = init_2d_map_from_file(input.clone());
    let n_threads = get_num_threads().unwrap_or(1);

    #[cfg(debug_assertions)] // check input
    {
        use honeycomb::prelude::OrbitPolicy;
        assert!(
            map.par_iter_faces()
                .all(|f| { map.orbit(OrbitPolicy::Face, f as DartIdType).count() == 3 }),
            "Input mesh isn't a triangle mesh"
        );
    }

    println!("| cut-edges benchmark");
    println!(
        "|-> input      : {} (hash: {input_hash:#0x})",
        input.to_str().unwrap()
    );
    println!("|-> backend    : {:?} with {n_threads} thread(s)", backend);
    println!("|-> target size: {target_length}");
    println!("|-> init time  : {}ms", init_time.as_millis());

    internals::cut_edges(
        &mut map,
        T::from(target_length).unwrap(),
        backend,
        n_threads,
    );

    finalize_2d(map, save);
}
