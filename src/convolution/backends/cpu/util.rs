use image::{GenericImageView, Pixel};

use crate::{convolution::strategy::ImagePixel, prelude::*};

/// Apply a convolution.
/// The weights are fetched from the given [`Kernel`].
/// Normalization is applied.
///
/// The provided 3x3 view into an image must be able to be
/// iterated over using 0..3 indexing in both coordinates,
/// and should result in reading the 3x3 neighbourhood
/// centered around the output pixel we're interested in.
pub fn do_convolve(
    kernel: Kernel,
    pixel: &mut ImagePixel,
    view_3x3: &dyn GenericImageView<Pixel = ImagePixel>,
) {
    for row in 0..3 {
        for col in 0..3 {
            pixel.apply2(
                &view_3x3.get_pixel(col, row),
                |output_channel, input_channel| {
                    output_channel + input_channel * kernel.weight(row as usize, col as usize)
                },
            );
        }
    }
    pixel.apply(|channel| channel * kernel.normalization())
}
