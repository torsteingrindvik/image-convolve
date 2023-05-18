use clap::Parser;
use image_convolve::{
    convolution::{
        backends::{
            cpu,
            gpu::{self, offscreen::context::GpuCtx},
        },
        strategy::{convolve, prepare},
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
    let (input, output, kernel) = (&args.input, &args.output, args.kernel);

    let image = prepare(input)?;

    match args.backend {
        Backend::SingleNestedLoops => {
            let backend = cpu::single::NestedLoops::from((image, kernel));
            convolve(backend, output)?;
        }
        Backend::SingleNestedIterators => {
            let backend = cpu::single::NestedIterators::from((image, kernel));
            convolve(backend, output)?;
        }
        Backend::MultiRayon => {
            let backend = cpu::multi::NestedIterators::from((image, kernel));
            convolve(backend, output)?;
        }
        Backend::GpuOffscreen => {
            let ctx = GpuCtx::new(image)?;
            let backend = gpu::offscreen::Offscreen::new(ctx, kernel)?;
            convolve(backend, output)?;
        }
    }

    Ok(())
}
