use std::{num::NonZero, path::PathBuf};

use clap::{Parser, Subcommand};

use applications::FileFormat;

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
    /// Input map as a VTK file
    #[arg(short, long, required(true))]
    pub input: PathBuf,
    /// Number of applications of the relaxation algorithm
    #[arg(long = "n-rounds", default_value_t = NonZero::new(100).unwrap())]
    pub n_rounds: NonZero<usize>,
    /// UNIMPLEMENTED - Use a Z-curve to sort vertices and reduce probability of conflict
    #[arg()]
    pub sort: bool,
    /// Serialize the map returned by the benchmark, if applicable
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
    #[command(subcommand)]
    pub command: Option<SmoothingType>,
}

#[derive(Subcommand)]
pub enum SmoothingType {
    /// Laplace smoothing
    Laplace {
        /// Scale coefficient
        #[arg(long, default_value_t = 0.5)]
        lambda: f64,
    },
    /// Taubin smoothing
    Taubin {
        /// Scale coefficient
        #[arg(long, default_value_t = 0.6307)]
        lambda: f64,
        /// Pass-band parameter
        #[arg(short, long("pass-band"), default_value_t = 0.1)]
        k: f64,
    },
}
