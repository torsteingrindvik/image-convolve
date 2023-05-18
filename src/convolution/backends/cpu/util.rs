use image::{DynamicImage, GenericImageView, Pixel, SubImage};

use crate::kernel::KernelImpl;

/// The type of image pixel we will be working with on the CPU.
pub type ImagePixel = image::Rgb<f32>;
/// The type of image we will be working with.
pub type Image = image::ImageBuffer<ImagePixel, Vec<f32>>;

pub(crate) struct ImageBuffers {
    pub input: Image,
    pub output: Image,
}

impl ImageBuffers {
    pub(crate) fn dimensions(&self) -> (usize, usize) {
        (self.input.width() as usize, self.input.height() as usize)
    }
}

impl ImageBuffers {
    pub(crate) fn new(input: DynamicImage) -> Self {
        let output = Image::new(input.width(), input.height());
        let input = input.to_rgb32f();

        Self { input, output }
    }
}

/// Apply a convolution.
/// The weights are fetched from the given [`Kernel`].
/// Normalization is applied.
///
/// The provided 3x3 view into an image must be able to be
/// iterated over using 0..3 indexing in both coordinates,
/// and should result in reading the 3x3 neighbourhood
/// centered around the output pixel we're interested in.
#[inline(always)]
pub fn do_convolve(
    kernel: KernelImpl,
    pixel: &mut ImagePixel,
    view_3x3: &dyn GenericImageView<Pixel = ImagePixel>,
) {
    for row in 0..3 {
        for col in 0..3 {
            unsafe {
                pixel.apply2(
                    // &view_3x3.get_pixel(col, row),
                    &view_3x3.unsafe_get_pixel(col, row),
                    |output_channel, input_channel| {
                        // output_channel + input_channel * kernel.weight(row as usize, col as usize)
                        output_channel
                            + input_channel * kernel.weights[col as usize + row as usize * 3]
                    },
                );
            }
        }
    }
    pixel.apply(|channel| channel * kernel.normalization)
}

pub type Kernel3x3<'i> = SubImage<&'i Image>;

/// Creates a view into an image of a 3x3 pixel area.
///
/// # Panics
///
/// If there isn't space to create the pixel area.
#[inline(always)]
pub fn view3x3(image: &Image, row: u32, column: u32) -> Kernel3x3 {
    debug_assert!(row != 0);
    debug_assert!(row < image.height() - 1);
    debug_assert!(column != 0);
    debug_assert!(column < image.width() - 1);

    image.view(column - 1, row - 1, 3, 3)
}
