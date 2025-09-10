use std::{num::NonZero, path::PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Honeycomb benchmarks binary
///
/// Each command of this binary correspond to a different (category of) benchmark(s). More
/// information about each is available using `hc-bench <COMMAND> --help`
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
    /// 3D incremental Delaunay triangulation of a box.
    DelaunayBox(DelaunayBoxArgs),
    /// `grisubal` kernel execution
    Grisubal(GrisubalArgs),
    /// Geometry capture, triangulation and remeshing kernel
    Remesh(RemeshArgs),
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
pub struct DelaunayBoxArgs {
    /// Number of cells along the X-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub lx: f64,
    /// Number of cells along the Y-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub ly: f64,
    /// Length of cells along the X-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub lz: f64,
    /// Number of points to insert
    #[arg(required(true))]
    pub n_points: NonZero<usize>,
    /// Number of points to insert
    #[arg(long("init"))]
    pub n_points_init: Option<NonZero<usize>>,
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
pub struct RemeshArgs {
    // -- capture args
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
    pub clip: Clip,

    // -- remeshing args
    /// Target value for edge length.
    #[arg(
        short('l'),
        long("target-length"),
        required(true),
        allow_negative_numbers(false)
    )]
    pub target_length: f64,
    /// Tolerance for target values. This should be between 0 and 1.
    #[arg(
        long("target-tolerance"),
        allow_negative_numbers(false),
        default_value_t = 0.2
    )]
    pub target_tolerance: f64,
    /// Maximum number of remeshing rounds. Less may be executed if early return is enabled.
    #[arg(
        long("n-rounds"),
        allow_negative_numbers(false),
        default_value_t = NonZero::new(100).unwrap()
    )]
    pub n_rounds: NonZero<usize>,
    /// Number of vertex relaxation rounds.
    #[arg(
        long("n-relax-rounds"),
        allow_negative_numbers(false),
        default_value_t = NonZero::new(5).unwrap()
    )]
    pub n_relax_rounds: NonZero<usize>,
    /// Enable early return in case target conditions are met within tolerance.
    #[arg(long = "enable-early-return")]
    pub enable_er: bool,
    /// Execution backend; number of threads used is determined using `std::thread::available_parallelism`.
    /// UNIMPLEMENTED
    #[arg(long, value_enum, default_value_t = Backend::RayonIter)]
    pub backend: Backend,
}

#[derive(Args)]
pub struct ShiftArgs {
    /// Input map as a VTK file
    #[arg(short, long, required(true))]
    pub input: PathBuf,
    /// Number of applications of the relaxation algorithm
    #[arg(long = "n-rounds", default_value_t = NonZero::new(100).unwrap())]
    pub n_rounds: NonZero<usize>,
    /// UNIMPLEMENTED - Use a partitioning algorithm to avoid conflicts between transactions
    #[arg(long = "no-conflict")]
    pub no_conflict: bool,
}
