mod cli;
mod internals;

use std::{io::Write, path::PathBuf};

use clap::Parser;
use honeycomb::prelude::CoordsFloat;
#[cfg(feature = "render")]
use honeycomb::render::render_2d_map;

use applications::{FileFormat, bind_rayon_threads};

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(cli.input, cli.algorithm, cli.save_as);
    } else {
        run_bench::<f64>(cli.input, cli.algorithm, cli.save_as);
    }
}

fn run_bench<T: CoordsFloat>(input: PathBuf, algorithm: cli::Algorithm, save: Option<FileFormat>) {
    let mut map = internals::init_2d_map::<T>(input);

    match algorithm {
        cli::Algorithm::EarClip => internals::earclip_cells(&mut map),
        cli::Algorithm::Fan => internals::fan_cells(&mut map),
    }

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
