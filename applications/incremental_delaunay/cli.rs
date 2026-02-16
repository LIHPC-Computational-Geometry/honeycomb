use std::num::NonZero;

use clap::Parser;

use applications::FileFormat;

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
    /// Number of the sampling domain along the X-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub lx: f64,
    /// Number of the sampling domain along the Y-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub ly: f64,
    /// Length of the sampling domain along the X-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub lz: f64,
    /// Number of points to insert
    #[arg(required(true))]
    pub n_points: NonZero<usize>,
    /// Seed for point campling
    #[arg(long("seed"))]
    pub seed: Option<u64>,
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}
