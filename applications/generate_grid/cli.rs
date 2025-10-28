use std::num::NonZero;

use clap::{Args, Parser, Subcommand};

use applications::FileFormat;

#[derive(Parser)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
    #[command(subcommand)]
    pub dim: Dim,
    /// Serialize the map returned by the benchmark
    #[arg(short, long("save-as"), value_enum, value_name("FORMAT"))]
    pub save_as: Option<FileFormat>,
    /// Execute benchmarks using `f32` instead of the default `f64`
    #[arg(long("simple-precision"))]
    pub simple_precision: bool,
    /// Execute benchmarks using an NVIDIA GPU via `cudarc` - requires `cudarc` feature
    #[arg(long)]
    pub gpu: bool,
}

#[derive(Subcommand)]
pub enum Dim {
    #[command(name = "2d")]
    /// Generate a 2D grid
    Two(Args2),
    #[command(name = "3d")]
    /// Generate a 3D grid
    Three(Args3),
}

#[derive(Args)]
pub struct Args2 {
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
    #[arg(long("split-in-tris"))]
    pub split: bool,
}

#[derive(Args)]
pub struct Args3 {
    /// Number of cells along the X-axis
    #[arg(required(true))]
    pub nx: NonZero<usize>,
    /// Number of cells along the Y-axis
    #[arg(required(true))]
    pub ny: NonZero<usize>,
    /// Length of cells along the Z-axis
    #[arg(required(true))]
    pub nz: NonZero<usize>,
    /// Length of cells along the X-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub lx: f64,
    /// Length of cells along the Y-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub ly: f64,
    /// Length of cells along the Z-axis
    #[arg(required(true), allow_negative_numbers(false))]
    pub lz: f64,
    /// If present, split diagonal according to the specified option
    #[arg(long("split-in-tets"))]
    pub split: bool,
}
