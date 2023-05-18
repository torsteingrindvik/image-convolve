use std::{fs::File, io::BufReader, path::Path};

use image::{
    codecs::{jpeg::JpegDecoder, png::PngDecoder},
    DynamicImage,
};
use tracing::info;

use crate::prelude::*;

/// The common strategy convolution "backends" should implement.
/// Backends will be driven by calling prepare, convolve, and finish in that order.
pub trait ConvolveStrategy {
    /// Prepare the input image for convolution with the given
    /// kernel.
    // fn prepare(&mut self, input: DynamicImage, kernel: Kernel) -> Result<()>;

    /// Apply the convolution on the input image, storing the results
    /// in an appropriate internal buffer.
    fn convolve(&mut self) -> Result<()>;

    /// Finish up by consuming any necessary buffers and producing the final image.
    fn finish(self) -> Result<DynamicImage>;
}

/// Prepares for convolution by creating an image buffer from the
/// input path and allocating an equally sized output image buffer for writing to.
pub fn prepare<P: AsRef<Path>>(input: P) -> Result<DynamicImage> {
    // Check that we're able to find the extensions and that they're equal
    info!("Reading extensions");
    let ext = extension(input.as_ref())?;

    // Read the input image
    // TODO: Either mmap or mmap via CLI flag?
    info!("Reading input file");
    let reader = BufReader::new(File::open(input)?);

    info!("Decoding input file");
    let image_input = get_dynamic_image(reader, &ext)?;

    // info!("Preparing output image buffer");
    // let image_output = Image::new(image_input.width(), image_input.height());

    Ok(image_input)
}

/// Convolve the input file by using the given backend.
pub fn convolve<Backend: ConvolveStrategy, P: AsRef<Path>>(
    mut backend: Backend,
    // input: P,
    output: P,
    // kernel: Kernel,
) -> Result<()> {
    // let image_input = prepare(input)?;
    // let mut backend = Backend::from((image_input, kernel));

    info!("Executing convolution");
    backend.convolve()?;

    info!("Finishing");
    let image_output = backend.finish()?.to_rgb8();

    info!("Saving result");
    image_output.save(output)?;

    Ok(())
}

// Check if input/output extensions match, error out if not.
// Stringify the extension for simplicity.
fn extension(p: &Path) -> Result<String> {
    if let Some(ext) = p.extension() {
        Ok(ext.to_string_lossy().to_string())
    } else {
        Err(Error::Usage(format!(
            "Cannot get extension from path `{}`",
            p.to_string_lossy()
        )))
    }
}

fn get_dynamic_image(reader: BufReader<File>, extension: &str) -> Result<DynamicImage> {
    fn png(reader: BufReader<File>) -> Result<DynamicImage> {
        let png = PngDecoder::new(reader)?;
        let dynamic_image = DynamicImage::from_decoder(png)?;

        Ok(dynamic_image)
    }

    fn jpeg(reader: BufReader<File>) -> Result<DynamicImage> {
        let png = JpegDecoder::new(reader)?;
        let dynamic_image = DynamicImage::from_decoder(png)?;

        Ok(dynamic_image)
    }

    match extension {
        "png" => png(reader),
        "jpg" | "jpeg" => jpeg(reader),
        _ => Err(Error::Limitation(format!(
            "Unsupported extension `{extension}`"
        ))),
    }
}
