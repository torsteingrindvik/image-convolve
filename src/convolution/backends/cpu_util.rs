use image::{GenericImageView, Pixel};

use crate::{convolution::strategy::ImagePixel, prelude::*};

/// Apply a kernel
pub fn do_convolve(
    kernel: Kernel,
    pixel: &mut ImagePixel,
    view: &dyn GenericImageView<Pixel = ImagePixel>,
) {
    for row in 0..3 {
        for col in 0..3 {
            pixel.apply2(
                &view.get_pixel(col, row),
                |output_channel, input_channel| {
                    output_channel + input_channel * kernel.weight(row as usize, col as usize)
                },
            );
        }
    }
    pixel.apply(|channel| channel * kernel.normalization())
}
