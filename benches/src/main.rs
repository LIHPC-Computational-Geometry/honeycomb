use clap::Parser;
use honeycomb::prelude::CMap2;

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
        todo!() // replace this block and the following with a macro-generated body
    } else {
        let map: CMap2<f64> = match cli.benches {
            Benches::Generate2dGrid(args) => bench_generate_2d_grid(args),
            Benches::CutEdges(args) => bench_cut_edges(args),
            Benches::Grisubal(args) => bench_grisubal(args),
            Benches::Shift(args) => bench_shift(args),
        };
        // all bench currently generate a map,
        // we may have to move this to match arms if this changes
        if let Some(f) = cli.save_as {
            match f {
                Format::Cmap => map.serialize("out.cmap"),
                Format::Vtk => map.to_vtk_binary("out.vtk"),
            }
        } else {
            std::hint::black_box(map);
        }
    }
}
