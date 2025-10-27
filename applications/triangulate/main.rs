mod cli;
mod internals;

use std::path::PathBuf;

use clap::Parser;
use honeycomb::prelude::CoordsFloat;

use applications::{FileFormat, bind_rayon_threads, finalize_2d, init_2d_map_from_file};

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(cli.input, cli.algorithm, cli.save_as);
    } else {
        run_bench::<f64>(cli.input, cli.algorithm, cli.save_as);
    }
}

fn run_bench<T: CoordsFloat>(input: PathBuf, algorithm: cli::Algorithm, save: Option<FileFormat>) {
    let (mut map, _, _) = init_2d_map_from_file::<T>(input);

    match algorithm {
        cli::Algorithm::EarClip => internals::earclip_cells(&mut map),
        cli::Algorithm::Fan => internals::fan_cells(&mut map),
    }

    finalize_2d(map, save);
}
