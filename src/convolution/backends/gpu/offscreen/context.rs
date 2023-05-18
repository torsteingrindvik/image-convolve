use std::sync::Arc;

use image::DynamicImage;
use tokio::runtime::Runtime;
use wgpu::{Adapter, Device, Queue, RenderPipeline};

use crate::prelude::*;

use super::{texture, FORMAT};

/// GPU data context.
/// Useful for benchmarks, since it allows setting up a GPU context (and related resources) once,
/// when benchmarks must run hundreds or thousands of times.
#[derive(Debug)]
pub struct GpuData {
    /// TODO
    pub device: wgpu::Device,
    /// TODO
    pub queue: wgpu::Queue,
    /// TODO
    // pub render_pipeline: wgpu::RenderPipeline,
    /// TODO
    pub diffuse_bind_group: wgpu::BindGroup,
    /// TODO
    pub render_texture: texture::RenderTexture,
    /// TODO
    pub output_gpu_buffer: texture::OutputBuffer,

    /// TODO
    pub shader: wgpu::ShaderModule,

    /// TODO
    pub render_pipeline_layout: wgpu::PipelineLayout,
}

/// A clonable context.
#[derive(Debug, Clone)]
pub struct GpuCtx {
    /// The inner data this clonable handle wraps.
    pub inner: Arc<GpuData>,
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

impl GpuCtx {
    /// TODO
    pub fn new(diffuse: DynamicImage) -> Result<Self> {
        Runtime::new()
            .unwrap()
            .block_on(async { Self::async_new(diffuse).await })
    }

    /// TODO
    pub fn render_pipeline(&self, kernel: Kernel) -> RenderPipeline {
        let fs_entry = frag_entry_point(kernel);

        self.inner
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("Render Pipeline: {fs_entry}")),
                layout: Some(&self.inner.render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.inner.shader,
                    entry_point: "vs_fullscreen",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.inner.shader,
                    entry_point: fs_entry,
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
            })
    }

    async fn async_new(diffuse: DynamicImage) -> Result<Self> {
        let (_adapter, device, queue) = prepare_wgpu().await?;

        let (diffuse_texture, render_texture, output_gpu_buffer) =
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

        Ok(Self {
            inner: Arc::new(GpuData {
                device,
                queue,
                diffuse_bind_group,
                render_texture,
                output_gpu_buffer,
                shader,
                render_pipeline_layout,
            }),
        })
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
