mod cli;
#[cfg(feature = "cuda")]
mod gpu;

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
                cli.gpu,
                cli.save_as,
            ),
            cli::Dim::Three(args) => run_bench_3d::<f32>(
                [args.nx.get(), args.ny.get(), args.nz.get()],
                [args.lx, args.ly, args.lz],
                args.split,
                cli.gpu,
                cli.save_as,
            ),
        }
    } else {
        if cli.gpu {
            unimplemented!("Double precision isn't implemented by our GPU routines")
        }
        match cli.dim {
            cli::Dim::Two(args) => run_bench_2d::<f64>(
                [args.nx.get(), args.ny.get()],
                [args.lx, args.ly],
                args.split,
                cli.gpu,
                cli.save_as,
            ),
            cli::Dim::Three(args) => run_bench_3d::<f64>(
                [args.nx.get(), args.ny.get(), args.nz.get()],
                [args.lx, args.ly, args.lz],
                args.split,
                cli.gpu,
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
    gpu: bool,
    save: Option<FileFormat>,
) {
    println!("| Grid generation benchmark");
    println!("|-> cell type : {}", if split { "tris" } else { "quads" });
    println!("|-> domain    : [0;{}]x[0;{}]", len_cells[0], len_cells[1]);
    println!("|-> dimensions: {}x{}", n_cells[0], n_cells[1]);
    println!("|-> # of cells: {}", n_cells[0] * n_cells[1]);
    println!(
        "|-> # of darts: {}",
        n_cells[0] * n_cells[1] * if split { 6 } else { 4 }
    );
    let instant = std::time::Instant::now();
    let map = if gpu {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cuda")] {
                gpu::build_2d(n_cells, len_cells, split).unwrap()
            } else {
                unimplemented!("E: the `--gpu` option requires the `cuda` feature to be enabled");
            }
        }
    } else {
        GridBuilder::<2, T>::default()
            .n_cells(n_cells)
            .len_per_cell(len_cells.map(|v| T::from(v).unwrap()))
            .split_cells(split)
            .build()
            .unwrap()
    };
    println!("build time: {:.3e}s", instant.elapsed().as_secs_f32());

    finalize_2d(map, save);
}

fn run_bench_3d<T: CoordsFloat>(
    n_cells: [usize; 3],
    len_cells: [f64; 3],
    split: bool,
    gpu: bool,
    save: Option<FileFormat>,
) {
    println!("| Grid generation benchmark");
    println!("|-> cell type : {}", if split { "tets" } else { "hexs" });
    println!(
        "|-> domain    : [0;{}]x[0;{}]x[0;{}]",
        len_cells[0], len_cells[1], len_cells[2]
    );
    println!(
        "|-> dimensions: {}x{}x{}",
        n_cells[0], n_cells[1], n_cells[2]
    );
    println!("|-> # of cells: {}", n_cells[0] * n_cells[1] * n_cells[2]);
    println!(
        "|-> # of darts: {}",
        n_cells[0] * n_cells[1] * n_cells[2] * if split { 60 } else { 24 }
    );
    let instant = std::time::Instant::now();
    let map = if gpu {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cuda")] {
                gpu::build_3d(n_cells, len_cells, split).unwrap()
            } else {
                unimplemented!("E: the `--gpu` option requires the `cuda` feature to be enabled");
            }
        }
    } else {
        GridBuilder::<3, T>::default()
            .n_cells(n_cells)
            .len_per_cell(len_cells.map(|v| T::from(v).unwrap()))
            .split_cells(split)
            .build()
            .unwrap()
    };
    println!("build time: {:.3e}s", instant.elapsed().as_secs_f32());

    finalize_3d(map, save);
}
