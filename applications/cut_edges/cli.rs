use std::path::PathBuf;

use clap::Parser;

use applications::{Backend, FileFormat};

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
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
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
}
