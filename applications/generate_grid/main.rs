mod cli;

use clap::Parser;
use honeycomb::prelude::{CoordsFloat, grid_generation::GridBuilder};

use applications::{FileFormat, bind_rayon_threads, finalize_2d, finalize_3d};

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        match cli.dim {
            cli::Dim::Two(args) => run_bench_2d::<f32>(
                [args.nx.get(), args.ny.get()],
                [args.lx, args.ly],
                args.split,
                cli.save_as,
            ),
            cli::Dim::Three(args) => run_bench_3d::<f32>(
                [args.nx.get(), args.ny.get(), args.nz.get()],
                [args.lx, args.ly, args.lz],
                args.split,
                cli.save_as,
            ),
        }
    } else {
        match cli.dim {
            cli::Dim::Two(args) => run_bench_2d::<f64>(
                [args.nx.get(), args.ny.get()],
                [args.lx, args.ly],
                args.split,
                cli.save_as,
            ),
            cli::Dim::Three(args) => run_bench_3d::<f64>(
                [args.nx.get(), args.ny.get(), args.nz.get()],
                [args.lx, args.ly, args.lz],
                args.split,
                cli.save_as,
            ),
        }
    }
}

// we can't use a const generic for dimension because constraints on these aren't supported yet,
// and the `build` method is only implemented for dim 2 and 3

fn run_bench_2d<T: CoordsFloat>(
    n_cells: [usize; 2],
    len_cells: [f64; 2],
    split: bool,
    save: Option<FileFormat>,
) {
    let map = GridBuilder::<2, T>::default()
        .n_cells(n_cells)
        .len_per_cell(len_cells.map(|v| T::from(v).unwrap()))
        .split_cells(split)
        .build()
        .unwrap();

    finalize_2d(map, save);
}

fn run_bench_3d<T: CoordsFloat>(
    n_cells: [usize; 3],
    len_cells: [f64; 3],
    split: bool,
    save: Option<FileFormat>,
) {
    let map = GridBuilder::<3, T>::default()
        .n_cells(n_cells)
        .len_per_cell(len_cells.map(|v| T::from(v).unwrap()))
        .split_cells(split)
        .build()
        .unwrap();

    finalize_3d(map, save);
}
