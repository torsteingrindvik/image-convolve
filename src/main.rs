use clap::Parser;
use image_convolve::prelude::*;
use tracing::info;

fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    info!(?args, "CLI");
}
