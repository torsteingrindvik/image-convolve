use image::{GenericImageView, Pixel};

use crate::convolution::util::view3x3;
use crate::prelude::*;

/// A naive CPU convolution strategy.
/// Iterates over pixels in a nested loop.
pub struct CpuNaive;

impl ConvolveStrategy for CpuNaive {
    fn convolve(input: Image, output: &mut Image, kernel: Kernel) -> Result<()> {
        let (width, height) = (input.width(), output.height());

        for row in 1..(height - 1) {
            for col in 1..(width - 1) {
                // Need to deref the view in order to get access to methods such as `get_pixel`.
                let kernel_view = &*view3x3(&input, row, col);

                let out = output.get_pixel_mut(col, row);

                for i in 0..3 {
                    for j in 0..3 {
                        out.apply2(&kernel_view.get_pixel(i, j), |out, other| out + other);
                    }
                }

                out.apply(|channel| channel / 9.)

                // let a = kernel_view.get_pixel(0, 0);

                // output[(col, row)] = input[(col, row)];
            }
        }

        Ok(())
    }
}
