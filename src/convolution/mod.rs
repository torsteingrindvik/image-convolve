use crate::prelude::*;

use std::path::Path;

/// Convolution using a naive byte-for-byte approach on the CPU.
pub mod cpu_naive;

/// Convolution using rayon for using all cores available.
pub mod cpu_rayon;

/// The common strategy convolution "backends" should implement.
pub trait ConvolveStrategy {
    /// Given an input image path and a desired output image path,
    /// perform convolution using the given [`Kernel`].
    fn convolve(input: &Path, output: &Path, kernel: Kernel) -> Result<()>;
}
