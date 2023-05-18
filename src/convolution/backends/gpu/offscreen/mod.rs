use self::context::GpuCtx;
use crate::prelude::*;
use image::{Pixel, Rgba, RgbaImage};
use std::iter;
use tokio::sync::oneshot;
use wgpu::RenderPipeline;

/// Context necessary for running GPU backends.
pub mod context;

pub(crate) mod texture;

pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// GPU offscreen convolution.
#[derive(Debug)]
pub struct Offscreen {
    ctx: context::GpuCtx,
    render_pipeline: RenderPipeline,
    output_cpu_buffer: RgbaImage,
}

impl ConvolveStrategy for Offscreen {
    fn convolve(&mut self) -> Result<()> {
        // Execute the pipeline on the GPU.
        let idx = self.render();

        // The rest is mapping the GPU buffer to CPU side and then
        // creating an image from it.

        let buffer_slice = self.ctx.inner.output_gpu_buffer.buffer.slice(..);
        let (tx, rx) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |res| tx.send(res).unwrap());

        self.ctx
            .inner
            .device
            .poll(wgpu::MaintainBase::WaitForSubmissionIndex(idx));

        rx.blocking_recv().unwrap().unwrap();

        let padded_buffer = buffer_slice.get_mapped_range();

        let out_rows = self.output_cpu_buffer.enumerate_rows_mut();
        let in_rows = padded_buffer.chunks(
            self.ctx
                .inner
                .output_gpu_buffer
                .dimensions
                .padded_bytes_per_row,
        );

        for ((_, buf_out), buf_in) in out_rows.zip(in_rows) {
            buf_out.for_each(|(col, _, pixel)| {
                let pos = col as usize * 4;
                *pixel = *Rgba::from_slice(&buf_in[pos..(pos + 4)]);
            });
        }

        drop(padded_buffer);
        self.ctx.inner.output_gpu_buffer.buffer.unmap();

        Ok(())
    }

    fn finish(self) -> Result<image::DynamicImage> {
        Ok(self.output_cpu_buffer.into())
    }
}

impl Offscreen {
    /// Create a new [`Offscreen`] instance with the given [`GpuCtx`] and [`Kernel`].
    pub fn new(context: GpuCtx, kernel: Kernel) -> Result<Self> {
        let dims = &context.inner.output_gpu_buffer.dimensions;
        let output_cpu_buffer = RgbaImage::new(dims.width as u32, dims.height as u32);

        let render_pipeline = context.render_pipeline(kernel);

        Ok(Self {
            ctx: context,
            output_cpu_buffer,
            render_pipeline,
        })
    }

    fn render(&mut self) -> wgpu::SubmissionIndex {
        let mut encoder =
            self.ctx
                .inner
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.ctx.inner.render_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.ctx.inner.diffuse_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            self.ctx.inner.render_texture.texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &self.ctx.inner.output_gpu_buffer.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    // This is where [`BufferDimensions`] comes in.
                    // Copy operations are particular about how many bytes each row contains,
                    // and we therefore might have padded rows here.
                    bytes_per_row: Some(
                        self.ctx
                            .inner
                            .output_gpu_buffer
                            .dimensions
                            .padded_bytes_per_row as u32,
                    ),
                    rows_per_image: None,
                },
            },
            self.ctx.inner.render_texture.extent,
        );

        self.ctx.inner.queue.submit(iter::once(encoder.finish()))
    }
}
