use std::path::PathBuf;

use clap::Parser;

use applications::{Clip, FileFormat};

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
    /// Input mesh as a VTK file
    #[arg(required(true))]
    pub input: PathBuf,
    /// Length of cells along the X-axis of the overlapping grid
    #[arg(required(true), allow_negative_numbers(false))]
    pub lx: f64,
    /// Length of cells along the Y-axis of the overlapping grid
    #[arg(required(true), allow_negative_numbers(false))]
    pub ly: f64,
    /// If present, clip cells on one side of the captured boundary
    #[arg(long, value_enum, value_name("SIDE"))]
    pub clip: Option<Clip>,
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}
