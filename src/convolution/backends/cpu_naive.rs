use image::RgbImage;

use crate::prelude::*;

/// A naive CPU convolution strategy.
/// Iterates over pixels in a nested loop.
pub struct CpuNaive;

impl ConvolveStrategy for CpuNaive {
    fn convolve(input: RgbImage, output: &mut RgbImage, _kernel: Kernel) -> Result<()> {
        let (width, height) = (input.width(), output.height());

        for row in 0..height {
            for col in 0..width {
                output[(col, row)] = input[(col, row)];
            }
        }

        Ok(())
    }
}
