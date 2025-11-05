use std::num::NonZero;

use clap::{Args, Parser};

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
    #[command(flatten)]
    pub alternate_init: Option<AlternateInit>,
    /// Seed for point campling
    #[arg(long("seed"))]
    pub seed: Option<u64>,
    /// Sort points to insert along a Z-curve - requires `spatial-sort` feature
    #[arg(long("enable-spatial-sort"))]
    pub sort: bool,
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct AlternateInit {
    /// Initialize the mesh with sequential point insertions
    #[arg(long("init-points"))]
    pub n_points_init: Option<NonZero<usize>>,
    /// Initialize the first triangulation from an existing mesh
    #[arg(long("init-file"))]
    pub file_init: Option<String>,
}
