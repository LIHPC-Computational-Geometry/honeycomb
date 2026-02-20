mod cli;
mod internals;

use clap::Parser;
use honeycomb::prelude::CoordsFloat;

use applications::{FileFormat, bind_rayon_threads, finalize_3d};

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(
            cli.lx,
            cli.ly,
            cli.lz,
            cli.n_points.get(),
            cli.seed.unwrap_or(123456789),
            cli.brio,
            cli.save_as,
        );
    } else {
        run_bench::<f64>(
            cli.lx,
            cli.ly,
            cli.lz,
            cli.n_points.get(),
            cli.seed.unwrap_or(123456789),
            cli.brio,
            cli.save_as,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn run_bench<T: CoordsFloat>(
    lx: f64,
    ly: f64,
    lz: f64,
    n_points: usize,
    seed: u64,
    probability: f64,
    save: Option<FileFormat>,
) {
    let map = internals::delaunay_box_3d::<T>(lx, ly, lz, n_points, seed, probability);

    finalize_3d(map, save);
}
