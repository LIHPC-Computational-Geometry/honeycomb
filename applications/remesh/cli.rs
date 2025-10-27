use std::{num::NonZero, path::PathBuf};

use applications::{Backend, Clip, FileFormat};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
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

    // -- common args
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}
