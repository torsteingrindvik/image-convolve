use std::ops::Range;

use image::{DynamicImage, GenericImageView};

use crate::kernel::KernelImpl;
use crate::prelude::*;

use super::util::{do_convolve, view3x3, ImageBuffers};

/// A straight forward CPU convolution strategy.
/// Iterates over pixels in a nested loop.
pub struct NestedLoops {
    buffers: ImageBuffers,
    kernel: KernelImpl,
    ranges: ConvolutionRanges,
}

impl From<(DynamicImage, Kernel)> for NestedLoops {
    fn from((input, kernel): (DynamicImage, Kernel)) -> Self {
        let (width, height) = input.dimensions();

        Self {
            buffers: ImageBuffers::new(input),
            kernel: kernel.into(),
            ranges: ConvolutionRanges::new(width, height),
        }
    }
}

impl ConvolveStrategy for NestedLoops {
    fn convolve(&mut self) -> Result<()> {
        for row in self.ranges.rows.clone() {
            for col in self.ranges.columns.clone() {
                // Need to deref the view in order to get access to methods such as `get_pixel`.
                let kernel_view = &*view3x3(&self.buffers.input, row, col);
                let pixel = self.buffers.output.get_pixel_mut(col, row);

                do_convolve(self.kernel, pixel, kernel_view);
            }
        }

        Ok(())
    }

    fn finish(self) -> Result<image::DynamicImage> {
        Ok(self.buffers.output.into())
    }
}

/// Uses a row iterator where each row iterator then does work on
/// each pixel.
pub struct NestedIterators {
    buffers: ImageBuffers,
    kernel: KernelImpl,
}

impl From<(DynamicImage, Kernel)> for NestedIterators {
    fn from((input, kernel): (DynamicImage, Kernel)) -> Self {
        Self {
            buffers: ImageBuffers::new(input),
            kernel: kernel.into(),
        }
    }
}

impl ConvolveStrategy for NestedIterators {
    fn convolve(&mut self) -> Result<()> {
        let (width, height) = self.buffers.dimensions();

        self.buffers
            .output
            .enumerate_rows_mut()
            .take(height - 1)
            .skip(1)
            .for_each(|(_, row_iter)| {
                row_iter
                    .take(width - 1)
                    .skip(1)
                    .for_each(|(col, row, pixel)| {
                        do_convolve(self.kernel, pixel, &*view3x3(&self.buffers.input, row, col))
                    })
            });

        Ok(())
    }

    fn finish(self) -> Result<DynamicImage> {
        Ok(self.buffers.output.into())
    }
}

/// For 3x3 convolution this provides iterators
/// which skip the first and last rows/columns such that we avoid
/// panics due to invalid access.
struct ConvolutionRanges {
    rows: Range<u32>,
    columns: Range<u32>,
}

impl ConvolutionRanges {
    fn new(width: u32, height: u32) -> Self {
        Self {
            rows: 1..(height - 1),
            columns: 1..(width - 1),
        }
    }
}
