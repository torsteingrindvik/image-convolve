use crate::convolution::util::view3x3;
use crate::prelude::*;

use super::cpu_util::do_convolve;

/// A straight forward CPU convolution strategy.
/// Iterates over pixels in a nested loop.
pub struct NestedLoops;

impl ConvolveStrategy for NestedLoops {
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

/// Uses a row iterator where each row iterator then does work on
/// each pixel.
pub struct NestedIterators;

impl ConvolveStrategy for NestedIterators {
    fn convolve(input: Image, output: &mut Image, kernel: Kernel) -> Result<()> {
        let width = input.width() as usize;
        let height = input.height() as usize;

        output
            .enumerate_rows_mut()
            .take(height - 1)
            .skip(1)
            .for_each(|(_, row_iter)| {
                row_iter
                    .take(width - 1)
                    .skip(1)
                    .for_each(|(col, row, pixel)| {
                        do_convolve(kernel, pixel, &*view3x3(&input, row, col))
                    })
            });

        Ok(())
    }
}
