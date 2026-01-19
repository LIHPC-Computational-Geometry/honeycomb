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
            cli.alternate_init,
            cli.seed.unwrap_or(123456789),
            cli.sort,
            cli.save_as,
        );
    } else {
        run_bench::<f64>(
            cli.lx,
            cli.ly,
            cli.lz,
            cli.n_points.get(),
            cli.alternate_init,
            cli.seed.unwrap_or(123456789),
            cli.sort,
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
    init: Option<cli::AlternateInit>,
    seed: u64,
    sort: bool,
    save: Option<FileFormat>,
) {
    let (n_points_init, file_init) = match init {
        Some(cli::AlternateInit {
            n_points_init,
            file_init,
        }) => (n_points_init.map(|v| v.get()).unwrap_or(0), file_init),
        None => (0, None),
    };

    let map =
        internals::delaunay_box_3d::<T>(lx, ly, lz, n_points, n_points_init, file_init, seed, sort);

    finalize_3d(map, save);
}
