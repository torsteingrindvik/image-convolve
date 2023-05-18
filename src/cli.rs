use std::path::PathBuf;

use crate::convolution::Backend;
use crate::prelude::*;
use clap::Parser;

/// Image convolution program.
/// The input image will be convolved and saved to the given output path.
#[derive(Parser, Debug)]
pub struct Cli {
    /// Path to input image
    #[arg(short, long)]
    pub input: PathBuf,

    /// Path to output image
    #[arg(short, long)]
    pub output: PathBuf,

    /// Kernel to apply to image
    #[arg(value_enum, short, long)]
    pub kernel: Kernel,

    /// Backend to use for convolution
    #[arg(value_enum, short, long)]
    pub backend: Backend,
}
