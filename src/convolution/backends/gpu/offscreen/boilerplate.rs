//! Adapted from (here)[https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/]

use image::{Pixel, Rgb};
use tokio::sync::oneshot;
use tracing::info;
use wgpu::{Adapter, Device, Queue};

use super::texture::{self, OutputBuffer, RenderTexture};
use super::FORMAT;
use crate::convolution::strategy::ImagePixel;
use crate::prelude::*;
use std::iter;
use std::time::Instant;

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
    render_texture: RenderTexture,
    output_buffer: OutputBuffer,
}

fn frag_entry_point(kernel: Kernel) -> &'static str {
    match kernel {
        Kernel::Identity => "fs_identity",
        Kernel::EdgeDetection1 => "fs_egde_detection1",
        Kernel::EdgeDetection2 => "fs_egde_detection2",
        Kernel::Sharpen => "fs_sharpen",
        Kernel::BoxBlur => "fs_box_blur",
        Kernel::GaussianBlur => "fs_gaussian_blur",
    }
}

pub async fn do_it(input: Image, output: &mut Image, kernel: Kernel) -> Result<()> {
    let mut state = State::new(input, kernel).await?;

    let start = Instant::now();
    info!("Next: Render {}us", (Instant::now() - start).as_micros());
    let submission_index = state.render().map_err(|e| Error::Gpu(format!("{e:?}")))?;
    let buf = &state.output_buffer;

    info!("Next: Await {}us", (Instant::now() - start).as_micros());
    let buffer_slice = buf.buffer.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |res| tx.send(res).unwrap());

    state
        .device
        .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));

    rx.await.unwrap().unwrap();

    info!("Next: Get map {}us", (Instant::now() - start).as_micros());
    let padded_buffer = buffer_slice.get_mapped_range();

    info!(
        "Next: Move buffer to CPU vec without padding {}us",
        (Instant::now() - start).as_micros()
    );

    // let mut out = Vec::with_capacity(buf.dimensions.unpadded_bytes_per_row * buf.dimensions.height);

    let out_rows = output.enumerate_rows_mut();
    let in_rows = padded_buffer.chunks(buf.dimensions.padded_bytes_per_row);

    // bytes per pixel
    let bpp = 4;

    out_rows
        .zip(in_rows)
        .for_each(|(((_row, row_iter), padded_row))| {
            row_iter.for_each(|(col, _row, pixel)| {
                let start = col as usize * bpp;
                let end = start + 3;

                // use image::c

                // *pixel = Rgb::<u8>::from_slice(&padded_row[start..end]).conv;
                // let p = Rgb::<u8>::from([1, 2, 3]);
                // let
                // pixel = ImagePixel::from_slice(&padded_row[0..4]);
            });
        });

    // padded_buffer
    //     .chunks(buf.dimensions.padded_bytes_per_row)
    //     .for_each(|row| {
    //         out.extend_from_slice(&row[..buf.dimensions.unpadded_bytes_per_row]);
    //     });

    drop(padded_buffer);
    buf.buffer.unmap();

    // info!(
    //     "Next: Store to disk {}us",
    //     (Instant::now() - start).as_micros()
    // );

    // image::save_buffer(
    //     "test.jpg",
    //     &out,
    //     buf.dimensions.width as u32,
    //     buf.dimensions.height as u32,
    //     image::ColorType::Rgba8,
    // )?;

    info!("Done {}us", (Instant::now() - start).as_micros());

    Ok(())
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

impl State {
    async fn new(diffuse: Image, kernel: Kernel) -> Result<Self> {
        let (_adapter, device, queue) = prepare_wgpu().await?;

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
            output_buffer,
            render_texture,
        })
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
                buffer: &self.output_buffer.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    // This is where [`BufferDimensions`] comes in.
                    // Copy operations are particular about how many bytes each row contains,
                    // and we therefore might have padded rows here.
                    bytes_per_row: Some(self.output_buffer.dimensions.padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            self.render_texture.extent,
        );

        let submission_index = self.queue.submit(iter::once(encoder.finish()));

        Ok(submission_index)
    }
}
