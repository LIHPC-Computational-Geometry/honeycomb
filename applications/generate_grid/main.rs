mod cli;

use std::io::Write;

use clap::Parser;
use honeycomb::prelude::{CoordsFloat, grid_generation::GridBuilder};
#[cfg(feature = "render")]
use honeycomb::render::{render_2d_map, render_3d_map};

use applications::{FileFormat, bind_rayon_threads};

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

    match save {
        Some(FileFormat::Cmap) => {
            // FIXME: update serialize sig
            let mut out = String::new();
            let mut file = std::fs::File::create("out.cmap").unwrap();
            map.serialize(&mut out);
            file.write_all(out.as_bytes()).unwrap();
        }
        Some(FileFormat::Vtk) => {
            let mut file = std::fs::File::create("out.vtk").unwrap();
            map.to_vtk_binary(&mut file);
        }
        None => {}
    }

    #[cfg(feature = "render")]
    {
        render_2d_map(&map);
    }
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

    match save {
        Some(FileFormat::Cmap) => {
            // FIXME: update serialize sig
            let mut out = String::new();
            let mut file = std::fs::File::create("out.cmap").unwrap();
            map.serialize(&mut out);
            file.write_all(out.as_bytes()).unwrap();
        }
        Some(FileFormat::Vtk) => {
            unimplemented!("E: VTK serialization isn't supported for 3-maps");
        }
        None => {}
    }

    #[cfg(feature = "render")]
    {
        render_3d_map(&map);
    }
}
