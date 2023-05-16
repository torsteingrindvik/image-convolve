use clap::Parser;
use image_convolve::convolution::backends;
use image_convolve::convolution::strategy::convolve;
use image_convolve::prelude::*;
use tracing::info;

fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    info!(?args, "CLI");

    convolve::<backends::cpu_naive::CpuNaive>(&args.input, &args.output, args.kernel)?;

    Ok(())
}
