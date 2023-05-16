use std::{ffi::OsStr, fs::File, io::BufReader, path::Path};

use image::{
    codecs::{jpeg::JpegDecoder, png::PngDecoder},
    DynamicImage,
};
use tracing::info;

use crate::prelude::*;

/// The type of  imagxelewe will working with.
pub type ImagePixel = image::Rgb<f32>;
/// The type of image we will working with.
pub type Image = image::ImageBuffer<ImagePixel, Vec<f32>>;

/// The common strategy convolution "backends" should implement.
pub trait ConvolveStrategy {
    /// Given an input image and an equally sized output image,
    /// perform convolution with the given [`Kernel`].
    fn convolve(input: Image, output: &mut Image, _kernel: Kernel) -> Result<()>;
}

/// Convolve the input file by using the given backend.
pub fn convolve<Backend: ConvolveStrategy>(
    input: &Path,
    output: &Path,
    kernel: Kernel,
) -> Result<()> {
    // Check that we're able to find the extensions and that they're equal
    info!("Reading extensions");
    let ext = check_ext(input, output)?;

    // Read the input image
    // TODO: Either mmap or mmap via CLI flag?

    info!("Reading input file");
    let reader = BufReader::new(File::open(input)?);

    info!("Decoding input file");
    let image_input: Image = get_dynamic_image(reader, &ext)?.into_rgb32f();

    info!("Preparing output");
    let mut image_output = Image::new(image_input.width(), image_input.height());

    info!("Executing convolution");
    Backend::convolve(image_input, &mut image_output, kernel)?;

    info!("Converting buffers to RGB8 for saving output");
    let image_output = DynamicImage::ImageRgb32F(image_output).to_rgb8();

    info!("Saving result");
    image_output.save(output)?;

    Ok(())
}

// Check if input/output extensions match, error out if not.
// Stringify the extension for simplicity.
fn check_ext(input: &Path, output: &Path) -> Result<String> {
    match (input.extension(), output.extension()) {
        (Some(i), Some(o)) => {
            // It's easier to deal with strings than OS strings.
            let simplify = |os_str: &OsStr| os_str.to_string_lossy().to_string();
            let i = simplify(i);
            let o = simplify(o);

            if i != o {
                return Err(Error::Usage(format!(
                    "Input extension `{i}` does not match output extension `{o}`",
                )));
            }

            Ok(i)
        }
        _ => Err(Error::Limitation(
            "Cannot guess input/output formats- please use file extensions".into(),
        )),
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
