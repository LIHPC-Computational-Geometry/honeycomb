use std::io::Write;

use clap::Parser;
use honeycomb::prelude::{CMap2, CoordsFloat};

use honeycomb_benches::{
    cli::{Benches, Cli, Format},
    cut_edges::bench_cut_edges,
    grid_gen::bench_generate_2d_grid,
    grisubal::bench_grisubal,
    shift::bench_shift,
};

fn main() {
    let cli = Cli::parse();

    if cli.simple_precision {
        run_benchmarks::<f32>(cli);
    } else {
        run_benchmarks::<f64>(cli);
    }
}

fn run_benchmarks<T: CoordsFloat>(cli: Cli) {
    let map: CMap2<T> = match cli.benches {
        Benches::Generate2dGrid(args) => bench_generate_2d_grid(args),
        Benches::CutEdges(args) => bench_cut_edges(args),
        Benches::Grisubal(args) => bench_grisubal(args),
        Benches::Shift(args) => bench_shift(args),
    };
    // all bench currently generate a map,
    // we may have to move this to match arms if this changes
    if let Some(f) = cli.save_as {
        match f {
            Format::Cmap => {
                // FIXME: update serialize sig
                let mut out = String::new();
                let mut file = std::fs::File::create("out.cmap").unwrap();
                map.serialize(&mut out);
                file.write_all(out.as_bytes()).unwrap();
            }
            Format::Vtk => {
                let mut file = std::fs::File::create("out.vtk").unwrap();
                map.to_vtk_binary(&mut file);
            }
        }
    } else {
        std::hint::black_box(map);
    }
}
