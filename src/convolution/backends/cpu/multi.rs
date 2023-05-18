use image::DynamicImage;
use rayon::prelude::*;

use crate::kernel::KernelImpl;
use crate::prelude::*;

use super::util::{do_convolve, view3x3, ImageBuffers};

/// Uses nested iterators, but runs in parallel at the row level.
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
            .par_bridge()
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
