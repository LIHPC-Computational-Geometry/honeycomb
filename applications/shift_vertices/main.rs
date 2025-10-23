mod cli;
mod internals;

use std::path::PathBuf;

use clap::Parser;
use honeycomb::prelude::CoordsFloat;

use applications::{
    FileFormat, bind_rayon_threads, finalize_2d, get_num_threads, init_2d_map_from_file,
    prof_start, prof_stop,
};

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(cli.input, cli.n_rounds.get(), cli.sort, cli.save_as);
    } else {
        run_bench::<f64>(cli.input, cli.n_rounds.get(), cli.sort, cli.save_as);
    }
}

fn run_bench<T: CoordsFloat>(
    input: PathBuf,
    n_rounds: usize,
    sort: bool,
    save: Option<FileFormat>,
) {
    let (map, input_hash, init_time) = init_2d_map_from_file::<T>(input.clone());

    println!("| shift benchmark");
    println!(
        "|-> input      : {} (hash: {input_hash:#0x})",
        input.to_str().unwrap()
    );
    println!(
        "|-> backend    : RayonIter with {} thread(s)",
        get_num_threads().unwrap_or(1)
    );
    println!("|-> # of rounds: {}", n_rounds);
    println!("|-+ init time  :");
    println!("| |->   map built in {}ms", init_time.as_millis());

    prof_start!("HCBENCH_SHIFT");
    let graph = internals::build_vertex_graph(&map, sort);
    internals::shift(&map, &graph, n_rounds);
    prof_stop!("HCBENCH_SHIFT");

    finalize_2d(map, save);
}
