use image::{GenericImageView, Pixel};

use crate::convolution::strategy::ImagePixel;
use crate::convolution::util::view3x3;
use crate::prelude::*;

/// A naive CPU convolution strategy.
/// Iterates over pixels in a nested loop.
pub struct CpuNaive;

/// Apply a kernel
fn do_convolve(
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

impl ConvolveStrategy for CpuNaive {
    fn convolve(input: Image, output: &mut Image, kernel: Kernel) -> Result<()> {
        let (width, height) = (input.width(), output.height());

        for row in 1..(height - 1) {
            for col in 1..(width - 1) {
                // Need to deref the view in order to get access to methods such as `get_pixel`.
                let kernel_view = &*view3x3(&input, row, col);
                let pixel = output.get_pixel_mut(col, row);

                do_convolve(kernel, pixel, kernel_view);
            }
        }

        Ok(())
    }
}
