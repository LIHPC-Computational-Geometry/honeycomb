mod cli;
mod internals;

use std::{io::Write, path::PathBuf};

use clap::Parser;
use honeycomb::prelude::{CMap2, CMapBuilder, CoordsFloat};
#[cfg(feature = "render")]
use honeycomb::render::render_2d_map;

use applications::{FileFormat, bind_rayon_threads, hash_file, prof_start, prof_stop};

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(cli.input, cli.n_rounds.get(), cli.sort, cli.save_as);
    } else {
        run_bench::<f64>(cli.input, cli.n_rounds.get(), cli.sort, cli.save_as);
    }
}

fn run_bench<T: CoordsFloat>(
    input: PathBuf,
    n_rounds: usize,
    sort: bool,
    save: Option<FileFormat>,
) {
    let instant = std::time::Instant::now();
    let input_map = input.to_str().unwrap();
    let input_hash = hash_file(input_map).unwrap();
    let map: CMap2<T> = if input_map.ends_with(".cmap") {
        CMapBuilder::<2>::from_cmap_file(input_map).build().unwrap()
    } else if input_map.ends_with(".vtk") {
        CMapBuilder::<2>::from_vtk_file(input_map).build().unwrap()
    } else {
        panic!(
            "E: Unknown file format; only .cmap or .vtk files are supported for map initialization"
        );
    };
    let build_time = instant.elapsed();
    let n_threads = rayon::current_num_threads();

    println!("| shift benchmark");
    println!("|-> input      : {input_map} (hash: {input_hash:#0x})");
    println!("|-> backend    : RayonIter with {n_threads} thread(s)",);
    println!("|-> # of rounds: {}", n_rounds);
    println!("|-+ init time  :");
    println!("| |->   map built in {}ms", build_time.as_millis());

    prof_start!("HCBENCH_SHIFT");
    let graph = internals::build_vertex_graph(&map, sort);
    internals::shift(&map, &graph, n_rounds);
    prof_stop!("HCBENCH_SHIFT");

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
        render_2d_map(map);
    }
}
