use std::path::PathBuf;

use clap::Parser;
use honeycomb::prelude::{
    CoordsFloat,
    grisubal::{Clip as KernelClip, grisubal},
};

use applications::{Clip as AppClip, FileFormat, bind_rayon_threads, finalize_2d};

mod cli;

fn main() {
    bind_rayon_threads!();

    let cli = cli::Cli::parse();

    if cli.simple_precision {
        run_bench::<f32>(cli.input, cli.lx, cli.ly, cli.clip, cli.save_as);
    } else {
        run_bench::<f64>(cli.input, cli.lx, cli.ly, cli.clip, cli.save_as);
    }
}

fn run_bench<T: CoordsFloat>(
    input: PathBuf,
    lx: f64,
    ly: f64,
    clip: Option<AppClip>,
    save: Option<FileFormat>,
) {
    let map = grisubal(
        input,
        [T::from(lx).unwrap(), T::from(ly).unwrap()],
        clip.map(KernelClip::from).unwrap_or_default(),
    )
    .unwrap(); // TODO: handle error properly

    finalize_2d(map, save);
}
