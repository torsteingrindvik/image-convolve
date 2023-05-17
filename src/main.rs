use clap::Parser;
use image_convolve::{
    convolution::{
        backends::{cpu_multi, cpu_single},
        strategy::convolve,
        Backend,
    },
    prelude::*,
};
use tracing::info;

fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    info!(?args, "CLI");

    // For brevity
    let (i, o, k) = (&args.input, &args.output, args.kernel);

    match args.backend {
        Backend::SingleNestedLoops => {
            convolve::<cpu_single::NestedLoops, _>(i, o, k)?;
        }
        Backend::SingleNestedIterators => {
            convolve::<cpu_single::NestedIterators, _>(i, o, k)?;
        }
        Backend::MultiRayon => {
            convolve::<cpu_multi::NestedIterators, _>(i, o, k)?;
        }
    }

    Ok(())
}
