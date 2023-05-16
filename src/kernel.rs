use clap::ValueEnum;

/// Pre-defined kernels.
/// See [Wikipedia](https://en.wikipedia.org/wiki/Kernel_(image_processing)).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Kernel {
    /// The identity operation.
    /// Should leave the image as-is.
    /// TODO: A good test would be using this and hashing the in/out to see that we're unaffected.
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
