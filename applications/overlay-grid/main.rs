use clap::Parser;
use honeycomb::prelude::CoordsFloat;
use std::path::PathBuf;

use applications::{FileFormat, bind_rayon_threads, finalize_2d};

mod cli;
mod internals;

fn main() {
    bind_rayon_threads!();
    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(cli.input, cli.nb_verts, cli.max_depth, cli.save_as);
    } else {
        run_bench::<f64>(cli.input, cli.nb_verts, cli.max_depth, cli.save_as);
    }
}

fn run_bench<T: CoordsFloat>(
    input: PathBuf,
    nb_verts: Option<usize>,
    max_depth: u32,
    save: Option<FileFormat>,
) {
    let map = internals::overlay_grid::<T>(input, [T::one(), T::one()], nb_verts, Some(max_depth))
        .unwrap();

    finalize_2d(map, save);
}
