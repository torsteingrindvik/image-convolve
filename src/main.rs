use clap::Parser;
use image_convolve::{
    convolution::{backends, strategy::convolve},
    prelude::*,
};
use tracing::info;

fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    info!(?args, "CLI");

    convolve::<_, backends::cpu_naive::CpuNaive>(&args.input, &args.output, args.kernel)?;

    Ok(())
}
