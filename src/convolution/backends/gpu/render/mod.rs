//! Adapted from (here)[https://github.com/gfx-rs/wgpu/blob/trunk/wgpu/examples/capture/main.rs].

use crate::prelude::*;
use std::mem::size_of;
use tokio::sync::oneshot;
use tracing::info;
use wgpu::{Adapter, Buffer, Device, Queue, SubmissionIndex};

pub(crate) mod boilerplate;
pub(crate) mod texture;

/// Make an image of this size
pub async fn run(png_output_path: &str, width: usize, height: usize) -> Result<()> {
    let (device, buffer, buffer_dimensions, submission_index) =
        create_red_image_with_dimensions(width, height).await?;

    create_png(
        png_output_path,
        device,
        buffer,
        &buffer_dimensions,
        submission_index,
    )
    .await?;

    Ok(())
}

/// Run the wgpu texture thing
pub async fn run2() -> Result<()> {
    boilerplate::do_it().await
    // boilerplate::run().await
}

async fn prepare_wgpu() -> Result<(Adapter, Device, Queue)> {
    let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or_else(|| Error::Gpu("No adapter".into()))?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|e| Error::Gpu(format!("{e:?}")))?;

    Ok((adapter, device, queue))
}

async fn create_red_image_with_dimensions(
    width: usize,
    height: usize,
) -> Result<(Device, Buffer, BufferDimensions, SubmissionIndex)> {
    let (_, device, queue) = prepare_wgpu().await?;

    let buffer_dimensions = BufferDimensions::new(width, height);
    info!(?buffer_dimensions, "Buffer");

    // The output buffer lets us retrieve the data as an array
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let texture_extent = wgpu::Extent3d {
        width: buffer_dimensions.width as u32,
        height: buffer_dimensions.height as u32,
        depth_or_array_layers: 1,
    };

    // The render pipeline renders data into this texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        label: None,
        view_formats: &[],
    });

    // Set the background to be red
    let command_buffer = {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        // Copy the data from the texture to the buffer
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            texture_extent,
        );

        encoder.finish()
    };

    let index = queue.submit(Some(command_buffer));
    Ok((device, output_buffer, buffer_dimensions, index))
}

async fn create_png(
    png_output_path: &str,
    device: Device,
    output_buffer: Buffer,
    buffer_dimensions: &BufferDimensions,
    submission_index: SubmissionIndex,
) -> Result<()> {
    let buffer_slice = output_buffer.slice(..);

    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = oneshot::channel();

    buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
        sender
            .send(v)
            .expect("Should be able to send async map result")
    });

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    //
    // We pass our submission index so we don't need to wait for any other possible submissions.
    device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));

    receiver
        .await
        .expect("Should be able to receive async map result")
        .map_err(|e| Error::Gpu(format!("Problem related to async mapping: {e:?}")))?;

    let padded_buffer = buffer_slice.get_mapped_range();

    let mut png_encoder = png::Encoder::new(
        std::fs::File::create(png_output_path).unwrap(),
        buffer_dimensions.width as u32,
        buffer_dimensions.height as u32,
    );
    png_encoder.set_depth(png::BitDepth::Eight);
    png_encoder.set_color(png::ColorType::Rgba);
    let mut png_writer = png_encoder
        .write_header()
        .unwrap()
        .into_stream_writer_with_size(buffer_dimensions.unpadded_bytes_per_row)
        .unwrap();

    // from the padded_buffer we write just the unpadded bytes into the image
    for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
        std::io::Write::write_all(
            &mut png_writer,
            &chunk[..buffer_dimensions.unpadded_bytes_per_row],
        )
        .unwrap();
    }
    png_writer.finish().unwrap();

    // With the current interface, we have to make sure all mapped views are
    // dropped before we unmap the buffer.
    drop(padded_buffer);

    output_buffer.unmap();
    Ok(())
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
