//! With help from (wgpu examples)[https://github.com/gfx-rs/wgpu/blob/trunk/wgpu/examples/capture/main.rs],
//! and (learn-wgpu)[https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/].

use self::texture::{OutputBuffer, RenderTexture};
use crate::prelude::*;
use image::{DynamicImage, Pixel, Rgba, RgbaImage};
use std::{iter, mem::size_of};
use tokio::{runtime::Runtime, sync::oneshot};
use wgpu::{Adapter, Device, Queue};

pub(crate) mod texture;
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// GPU offscreen convolution.
#[derive(Debug)]
pub struct Offscreen {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
    render_texture: RenderTexture,
    output_gpu_buffer: OutputBuffer,
    output_cpu_buffer: RgbaImage,
}

impl From<(DynamicImage, Kernel)> for Offscreen {
    fn from((input, kernel): (DynamicImage, Kernel)) -> Self {
        Offscreen::new(input, kernel).unwrap()
    }
}

impl ConvolveStrategy for Offscreen {
    fn convolve(&mut self) -> Result<()> {
        let idx = self.render().unwrap();

        let buffer_slice = self.output_gpu_buffer.buffer.slice(..);
        let (tx, rx) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |res| tx.send(res).unwrap());

        self.device
            .poll(wgpu::MaintainBase::WaitForSubmissionIndex(idx));

        rx.blocking_recv().unwrap().unwrap();

        let padded_buffer = buffer_slice.get_mapped_range();

        let out_rows = self.output_cpu_buffer.enumerate_rows_mut();
        let in_rows = padded_buffer.chunks(self.output_gpu_buffer.dimensions.padded_bytes_per_row);

        for ((_, buf_out), buf_in) in out_rows.zip(in_rows) {
            buf_out.for_each(|(col, _, pixel)| {
                let pos = col as usize * 4;
                *pixel = *Rgba::from_slice(&buf_in[pos..(pos + 4)]);
            });
        }

        drop(padded_buffer);
        self.output_gpu_buffer.buffer.unmap();

        Ok(())
    }

    fn finish(self) -> Result<image::DynamicImage> {
        Ok(self.output_cpu_buffer.into())
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

fn frag_entry_point(kernel: Kernel) -> &'static str {
    match kernel {
        Kernel::Identity => "fs_identity",
        Kernel::EdgeDetection1 => "fs_edge_detection1",
        Kernel::EdgeDetection2 => "fs_edge_detection2",
        Kernel::Sharpen => "fs_sharpen",
        Kernel::BoxBlur => "fs_box_blur",
        Kernel::GaussianBlur => "fs_gaussian_blur",
    }
}

async fn prepare_wgpu() -> Result<(Adapter, Device, Queue)> {
    let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .ok_or_else(|| Error::Gpu("No adapter".into()))?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|e| Error::Gpu(format!("{e:?}")))?;

    Ok((adapter, device, queue))
}

impl Offscreen {
    async fn async_new(diffuse: DynamicImage, kernel: Kernel) -> Result<Offscreen> {
        let (_adapter, device, queue) = prepare_wgpu().await?;

        let output_cpu_buffer = RgbaImage::new(diffuse.width(), diffuse.height());

        let (diffuse_texture, render_texture, output_buffer) =
            texture::prepare(&device, &queue, diffuse)?;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_fullscreen",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: frag_entry_point(kernel),
                targets: &[Some(wgpu::ColorTargetState {
                    format: FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            device,
            queue,
            render_pipeline,
            diffuse_bind_group,
            output_gpu_buffer: output_buffer,
            render_texture,
            output_cpu_buffer,
        })
    }

    // Sync version of new creates a temporary tokio runtime
    // and drives the future to completion.
    fn new(diffuse: DynamicImage, kernel: Kernel) -> Result<Self> {
        // Handle::current().block_on(async { Self::async_new(diffuse, kernel).await })
        Runtime::new()
            .unwrap()
            .block_on(async { Self::async_new(diffuse, kernel).await })
    }

    fn render(&mut self) -> std::result::Result<wgpu::SubmissionIndex, wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            self.render_texture.texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &self.output_gpu_buffer.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    // This is where [`BufferDimensions`] comes in.
                    // Copy operations are particular about how many bytes each row contains,
                    // and we therefore might have padded rows here.
                    bytes_per_row: Some(
                        self.output_gpu_buffer.dimensions.padded_bytes_per_row as u32,
                    ),
                    rows_per_image: None,
                },
            },
            self.render_texture.extent,
        );

        let submission_index = self.queue.submit(iter::once(encoder.finish()));

        Ok(submission_index)
    }
}
