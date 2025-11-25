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
        run_bench::<f32>(
            cli.input,
            cli.n_rounds.get(),
            cli.sort,
            cli.save_as,
            cli.command,
        );
    } else {
        run_bench::<f64>(
            cli.input,
            cli.n_rounds.get(),
            cli.sort,
            cli.save_as,
            cli.command,
        );
    }
}

fn run_bench<T: CoordsFloat>(
    input: PathBuf,
    n_rounds: usize,
    sort: bool,
    save: Option<FileFormat>,
    smoothing: Option<cli::SmoothingType>,
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

    match smoothing {
        None => {
            println!("|-> smoothing  : neighbor average",);
        }
        Some(cli::SmoothingType::Laplace { lambda }) => {
            println!("|-> smoothing  : Laplace");
            println!("|-> scale (λ)  : {}", lambda);
            assert!(
                lambda > 0.0 && lambda < 1.0,
                "lambda must verify `0 < lambda < 1`"
            );
        }
        Some(cli::SmoothingType::Taubin { lambda, k }) => {
            println!("|-> smoothing  : Taubin");
            println!("|-> scale (λ)  : {lambda}");
            println!("|-> pass (k)   : {k}");
            assert!(
                lambda > 0.0 && lambda < 1.0,
                "lambda must verify `0 < lambda < 1`"
            );
        }
    }
    println!("|-+ init time  :");
    println!("| |->   map built in {}ms", init_time.as_millis());

    prof_start!("HCBENCH_SHIFT");
    let graph = internals::build_vertex_graph(&map, sort);
    match smoothing {
        None => {
            internals::shift(&map, &graph, n_rounds);
        }
        Some(cli::SmoothingType::Laplace { lambda }) => {
            internals::laplace(&map, &graph, n_rounds, T::from(lambda).unwrap());
        }
        Some(cli::SmoothingType::Taubin { lambda, k }) => {
            internals::taubin(
                &map,
                &graph,
                n_rounds,
                T::from(lambda).unwrap(),
                T::from(k).unwrap(),
            );
        }
    }
    prof_stop!("HCBENCH_SHIFT");

    finalize_2d(map, save);
}
