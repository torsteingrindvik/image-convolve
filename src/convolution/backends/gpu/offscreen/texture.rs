use crate::prelude::*;
use image::{DynamicImage, GenericImageView};
use super::{FORMAT};

pub(crate) fn prepare(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img: DynamicImage,
) -> Result<(DiffuseTexture, RenderTexture, OutputBuffer)> {
    let render_texture = RenderTexture::from_image(device, &img)?;
    let output_buffer = OutputBuffer::from_image(device, &img)?;
    let texture = DiffuseTexture::from_image(device, queue, img, None)?;

    Ok((texture, render_texture, output_buffer))
}

#[derive(Debug)]
pub struct RenderTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub extent: wgpu::Extent3d,
}

impl RenderTexture {
    pub fn from_image(device: &wgpu::Device, img: &DynamicImage) -> Result<Self> {
        let (width, height) = img.dimensions();

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self { texture, extent, view })
    }
}

#[derive(Debug)]
pub struct OutputBuffer {
    pub buffer: wgpu::Buffer,
    pub dimensions: BufferDimensions,
}

impl OutputBuffer {
    pub fn from_image(device: &wgpu::Device, img: &DynamicImage) -> Result<Self> {
        let (width, height) = img.dimensions();

        let dimensions = BufferDimensions::new(width as usize, height as usize);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Offline Buffer"),
            size: (dimensions.padded_bytes_per_row * dimensions.height) as u64,
            usage: 
            // We want to be able to map this for reading on CPU side
            wgpu::BufferUsages::MAP_READ 
            // We also want to be able to use it as the destination of a texture copy (the offline render)
            | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(OutputBuffer { buffer, dimensions })
    }
}

pub struct DiffuseTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl DiffuseTexture {
    /// Given a [`DynamicImage`], prepare a 
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let (width, height) = img.dimensions();

        let rgba = img.to_rgba8();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            // Need to be able to use this in a shader (by binding), as well as using `write_texture` on it to load data.
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,

                // Due to the Rgba8 format each pixel is 4 bytes wide.
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // Use nearest everywhere.
            // We don't want to affect results by doing filtering.
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}

/// With help from (wgpu examples)[https://github.com/gfx-rs/wgpu/blob/trunk/wgpu/examples/capture/main.rs].
/// It is a WebGPU requirement that ImageCopyBuffer.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
/// So we calculate padded_bytes_per_row by rounding unpadded_bytes_per_row
/// up to the next multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
/// https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
#[derive(Debug)]
pub struct BufferDimensions {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        // RGBA spread out like [u8, u8, u8, u8], same size as a u32.
        let bytes_per_pixel = std::mem::size_of::<u32>();
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
