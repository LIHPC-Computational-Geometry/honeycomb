use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use applications::FileFormat;

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
    /// Input map as a VTK file
    #[arg(short, long, required(true))]
    pub input: PathBuf,
    /// Triangulation algorithm
    #[arg(short, long, value_enum)]
    pub algorithm: Algorithm,
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Algorithm {
    EarClip,
    Fan,
}
