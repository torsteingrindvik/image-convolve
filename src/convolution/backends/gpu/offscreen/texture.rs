//! Adapted from (here)[https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/]

use crate::prelude::*;

use image::{DynamicImage, GenericImageView};

use super::{BufferDimensions, FORMAT};

pub fn prepare(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img: DynamicImage,
) -> Result<(DiffuseTexture, RenderTexture, OutputBuffer)> {
    let render_texture = RenderTexture::from_image(device, &img)?;
    let output_buffer = OutputBuffer::from_image(device, &img)?;
    let texture = DiffuseTexture::from_image(device, queue, img, None)?;

    Ok((texture, render_texture, output_buffer))
}

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
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let (width, height) = img.dimensions();

        // TODO: This is probably expensive?
        let rgba = DynamicImage::ImageRgb32F(img).into_rgba8();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
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
