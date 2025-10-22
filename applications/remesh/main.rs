mod cli;
mod internals;

use std::{io::Write, path::PathBuf};

use clap::Parser;
use honeycomb::prelude::CoordsFloat;
#[cfg(feature = "render")]
use honeycomb::render::render_2d_map;

use applications::{Clip as AppClip, FileFormat, bind_rayon_threads};

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
    // capture
    let mut map = internals::generate_first_mesh(
        input,
        target_length,
        lens.map(|v| T::from(v).unwrap()),
        clip,
    );

    // remesh
    internals::remesh(
        &mut map,
        n_rounds,
        n_relax_rounds,
        target_length,
        target_tolerance,
        enable_early_ret,
    );

    // finalize
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
