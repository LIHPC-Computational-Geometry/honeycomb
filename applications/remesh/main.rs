mod cli;
mod internals;

use std::path::PathBuf;

use clap::Parser;
use honeycomb::prelude::CoordsFloat;

use applications::{Clip as AppClip, FileFormat, bind_rayon_threads, finalize_2d};

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(
            cli.input,
            [cli.lx, cli.ly],
            cli.clip,
            cli.n_rounds.get(),
            cli.n_relax_rounds.get(),
            cli.target_length,
            cli.target_tolerance,
            cli.enable_er,
            cli.save_as,
        );
    } else {
        run_bench::<f64>(
            cli.input,
            [cli.lx, cli.ly],
            cli.clip,
            cli.n_rounds.get(),
            cli.n_relax_rounds.get(),
            cli.target_length,
            cli.target_tolerance,
            cli.enable_er,
            cli.save_as,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn run_bench<T: CoordsFloat>(
    input: PathBuf,
    lens: [f64; 2],
    clip: AppClip,
    n_rounds: usize,
    n_relax_rounds: usize,
    target_length: f64,
    target_tolerance: f64,
    enable_early_ret: bool,
    save: Option<FileFormat>,
) {
    let mut map = internals::generate_first_mesh(
        input,
        target_length,
        lens.map(|v| T::from(v).unwrap()),
        clip,
    );

    internals::remesh(
        &mut map,
        n_rounds,
        n_relax_rounds,
        target_length,
        target_tolerance,
        enable_early_ret,
    );

    finalize_2d(map, save);
}
