use clap::Parser;
use image_convolve::{
    convolution::{backends::cpu, strategy::convolve, Backend},
    prelude::*,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    info!(?args, "CLI");

    // tmp
    image_convolve::convolution::backends::gpu::render::run2().await?;

    // For brevity
    let (i, o, k) = (&args.input, &args.output, args.kernel);

    match args.backend {
        Backend::SingleNestedLoops => {
            convolve::<cpu::single::NestedLoops, _>(i, o, k)?;
        }
        Backend::SingleNestedIterators => {
            convolve::<cpu::single::NestedIterators, _>(i, o, k)?;
        }
        Backend::MultiRayon => {
            convolve::<cpu::multi::NestedIterators, _>(i, o, k)?;
        }
    }

    // tmp
    image_convolve::convolution::backends::gpu::render::run("test.png", 100, 100).await?;

    Ok(())
}
