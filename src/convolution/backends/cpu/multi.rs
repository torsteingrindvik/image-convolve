use rayon::prelude::*;

use crate::convolution::util::view3x3;
use crate::prelude::*;

use super::util::do_convolve;

/// Uses nested iterators, but runs in parallel at the row level.
pub struct NestedIterators;

impl ConvolveStrategy for NestedIterators {
    fn convolve(input: Image, output: &mut Image, kernel: Kernel) -> Result<()> {
        let width = input.width() as usize;
        let height = input.height() as usize;

        output
            .enumerate_rows_mut()
            .take(height - 1)
            .skip(1)
            .par_bridge()
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
