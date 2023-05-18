use std::fmt::Display;

use clap::ValueEnum;

/// Pre-defined kernels.
/// See [Wikipedia](https://en.wikipedia.org/wiki/Kernel_(image_processing)).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Kernel {
    /// The identity operation.
    Identity,

    /// Edge detection version 1.
    EdgeDetection1,

    /// Edge detection version 2.
    EdgeDetection2,

    /// Sharpening.
    Sharpen,

    /// Box blur.
    BoxBlur,

    /// Gaussian blur.
    GaussianBlur,
}

impl Display for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A kernel with its associated weights an normalization factor.
#[derive(Debug, Copy, Clone)]
pub struct KernelImpl {
    /// Weights from top-left to bottom-right.
    pub weights: [f32; 9],

    /// Normalization.
    pub normalization: f32,
}

impl From<Kernel> for KernelImpl {
    fn from(kernel: Kernel) -> Self {
        Self {
            weights: *kernel.matrix(),
            normalization: kernel.normalization(),
        }
    }
}

impl Kernel {
    /// The matrix with weights for the given kernel.
    pub const fn matrix(&self) -> &'static [f32; 9] {
        match self {
            Kernel::Identity => &[0., 0., 0., 0., 1., 0., 0., 0., 0.],
            Kernel::EdgeDetection1 => &[0., -1., 0., -1., 4., -1., 0., -1., 0.],
            Kernel::EdgeDetection2 => &[-1., -1., -1., -1., 8., -1., -1., -1., -1.],
            Kernel::Sharpen => &[0., -1., -0., -1., 5., -1., 0., -1., 0.],
            Kernel::BoxBlur => &[1., 1., 1., 1., 1., 1., 1., 1., 1.],
            Kernel::GaussianBlur => &[1., 2., 1., 2., 4., 2., 1., 2., 1.],
        }
    }

    /// Get the matrix weight for the kernel at the given row, column.
    pub const fn weight(&self, row: usize, column: usize) -> f32 {
        self.matrix()[column + row * 3]
    }

    /// Get the normalization factor for the given kernel.
    pub const fn normalization(&self) -> f32 {
        match self {
            Kernel::Identity
            | Kernel::EdgeDetection1
            | Kernel::EdgeDetection2
            | Kernel::Sharpen => 1.,

            Kernel::BoxBlur => 0.111_111_11, // 1/9
            Kernel::GaussianBlur => 0.0625,  //  1/16
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::Kernel;

    #[test]
    fn weight_access() {
        assert_eq!(Kernel::EdgeDetection1.weight(1, 1), 4.);
        assert_eq!(Kernel::EdgeDetection2.weight(1, 1), 8.);
        assert_eq!(Kernel::GaussianBlur.weight(0, 0), 1.);
        assert_eq!(Kernel::GaussianBlur.weight(1, 0), 2.);
    }
}
