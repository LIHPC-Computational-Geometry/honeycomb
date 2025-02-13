use std::{num::NonZero, path::PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Honeycomb benchmarks binary
///
/// Each command of this binary correspond to a (category of) benchmark(s). More information
/// about each is available using `hc-bench <COMMAND> --help`
#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
    #[command(subcommand)]
    pub benches: Benches,
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<Format>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    Cmap,
    Vtk,
}

#[derive(Subcommand)]
pub enum Benches {
    /// 2D grid generation using `CMapBuilder` and `GridDescriptor`
    #[command(name = "generate-2d-grid")]
    Generate2dGrid(Generate2dGridArgs),
    /// Edge size reduction in triangular meshes using vertex/edge insertions
    CutEdges(CutEdgesArgs),
    /// `grisubal` kernel execution
    Grisubal(GrisubalArgs),
    /// Simple vertex relaxation routine
    Shift(ShiftArgs),
}

#[derive(Args)]
pub struct Generate2dGridArgs {
    /// Number of cells along the X-axis
    #[arg(required(true))]
    pub nx: NonZero<usize>,
    /// Number of cells along the Y-axis
    #[arg(required(true))]
    pub ny: NonZero<usize>,
    /// Length of cells along the X-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub lx: f64,
    /// Length of cells along the Y-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub ly: f64,
    /// If present, split diagonal according to the specified option
    #[arg(short, long, value_enum)]
    pub split: Option<Split>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Split {
    Uniform,
    Random,
}

#[derive(Args)]
pub struct CutEdgesArgs {
    /// Input map as a VTK file
    #[arg(short, long, required(true))]
    pub input: PathBuf,
    /// Execution backend; number of threads used is determined using `std::thread::available_parallelism`
    #[arg(long, value_enum, default_value_t = Backend::StdThreads)]
    pub backend: Backend,
    /// Target threshold for edge length; any edge equal or above is split in half
    #[arg(
        short('l'),
        long("target-length"),
        required(true),
        allow_negative_numbers(false)
    )]
    pub target_length: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Backend {
    RayonIter,
    RayonChunks,
    StdThreads,
}

#[derive(Args)]
pub struct GrisubalArgs {
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
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Clip {
    Left,
    Right,
}

#[derive(Args)]
pub struct ShiftArgs {
    /// Input map as a VTK file
    #[arg(short, long, required(true))]
    pub input: PathBuf,
    /// Number of applications of the relaxation algorithm
    #[arg(long = "n-rounds", default_value_t = 100)]
    pub n_rounds: usize,
    /// UNIMPLEMENTED - Use a partitioning algorithm to avoid conflicts between transactions
    #[arg(long = "no-conflict")]
    pub no_conflict: bool,
}
