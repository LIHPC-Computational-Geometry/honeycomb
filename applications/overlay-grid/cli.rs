use std::path::PathBuf;

use applications::FileFormat;
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Input mesh as a VTK file
    #[arg(
        long,
        short = 'i',
        conflicts_with = "nb_verts",
        default_value = "random"
    )]
    pub input: PathBuf,
    /// Number of vertices to generate for using random samples instead of an input file
    #[arg(long, short = 'n', conflicts_with = "input", default_value = "100")]
    pub nb_verts: Option<usize>,
    /// Maximum refinement depth
    #[arg(long, short = 'd', default_value = "8")]
    pub max_depth: u32,
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}
