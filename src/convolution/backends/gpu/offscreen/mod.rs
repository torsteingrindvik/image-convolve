//! Adapted from (here)[https://github.com/gfx-rs/wgpu/blob/trunk/wgpu/examples/capture/main.rs].

use image::DynamicImage;

use crate::prelude::*;
use std::mem::size_of;
pub(crate) mod boilerplate;
pub(crate) mod texture;

pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Run the wgpu texture thing
fn run2(image: Image, output: &mut Image, kernel: Kernel) -> Result<()> {
    use tokio::runtime::Runtime;

    // Create the runtime
    let rt = Runtime::new().unwrap();

    // Execute the future, blocking the current thread until completion
    rt.block_on(async {
        println!("hello");
        boilerplate::do_it(image, output, kernel).await
    })
}

/// GPU offscreen convolution.
#[derive(Debug, Default)]
pub struct Offscreen;

impl From<(DynamicImage, Kernel)> for Offscreen {
    fn from((input, kernel): (DynamicImage, Kernel)) -> Self {
        todo!();
    }
}

impl ConvolveStrategy for Offscreen {
    fn convolve(&mut self) -> Result<()> {
        // run2(input, output, kernel)
        todo!()
    }

    fn finish(self) -> Result<image::DynamicImage> {
        todo!()
    }
}

/// From `wgpu`:
/// It is a WebGPU requirement that ImageCopyBuffer.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
/// So we calculate padded_bytes_per_row by rounding unpadded_bytes_per_row
/// up to the next multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
/// https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
#[derive(Debug)]
pub struct BufferDimensions {
    width: usize,
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        // RGBA spread out like [u8, u8, u8, u8], same size as a u32.
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;

        // Right now, this number is 256 bytes.
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;

        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}
