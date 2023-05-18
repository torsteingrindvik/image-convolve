use std::path::Path;

use image::DynamicImage;
use tracing::info;

use crate::prelude::*;

/// The common strategy convolution "backends" should implement.
/// Backends will be driven by calling prepare, convolve, and finish in that order.
pub trait ConvolveStrategy {
    /// Apply the convolution on the input image, storing the results
    /// in an appropriate internal buffer.
    fn convolve(&mut self) -> Result<()>;

    /// Finish up by consuming any necessary buffers and producing the final image.
    fn finish(self) -> Result<DynamicImage>;
}

/// Prepares for convolution by creating an image buffer from the
/// input path and allocating an equally sized output image buffer for writing to.
pub fn prepare<P: AsRef<Path>>(input: P) -> Result<DynamicImage> {
    // Note: Could we speed things up with mmap?
    Ok(image::io::Reader::open(input)?.decode()?)
}

/// Convolve the input file by using the given backend.
pub fn convolve<Backend: ConvolveStrategy, P: AsRef<Path>>(
    mut backend: Backend,
    output: P,
) -> Result<()> {
    info!("Executing convolution");
    backend.convolve()?;

    info!("Finishing");
    let image_output = backend.finish()?.to_rgb8();

    info!("Saving result");
    image_output.save(output)?;

    Ok(())
}
