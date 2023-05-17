//! Adapted from (here)[https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/]

use tokio::sync::oneshot;
use tracing::info;

use super::texture::{self, DiffuseTexture, OutputBuffer, RenderTexture};
use crate::prelude::*;
use std::fs::File;
use std::io::Write;
use std::iter;
use std::time::Instant;

struct State {
    // surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    // config: wgpu::SurfaceConfiguration,
    // size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    #[allow(dead_code)]
    diffuse_texture: DiffuseTexture,
    diffuse_bind_group: wgpu::BindGroup,

    render_texture: RenderTexture,

    // window: Window,
    output_buffer: OutputBuffer,
    // render_timestamp: Instant,
}

pub async fn do_it() -> Result<()> {
    // let i = |msg| info!("{msg} {:?}", (start - Instant::now()).as_micros());

    // i("next: state");
    // info!("Next: State {}ms", (Instant::now() - start).as_millis());
    let mut state = State::new().await?;

    let start = Instant::now();
    info!("Next: Render {}us", (Instant::now() - start).as_micros());
    let submission_index = state.render().map_err(|e| Error::Gpu(format!("{e:?}")))?;

    info!("Next: Await {}us", (Instant::now() - start).as_micros());
    let buffer_slice = state.output_buffer.buffer.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |res| tx.send(res).unwrap());

    state
        .device
        .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));

    rx.await.unwrap().unwrap();

    info!("Next: Get map {}us", (Instant::now() - start).as_micros());
    let padded_buffer = buffer_slice.get_mapped_range();

    info!(
        "Next: Create file {}us",
        (Instant::now() - start).as_micros()
    );
    let f = File::create("test.png").unwrap();

    info!(
        "Next: Encode setup {}us",
        (Instant::now() - start).as_micros()
    );
    let mut png_encoder = png::Encoder::new(
        f,
        state.output_buffer.dimensions.width as u32,
        state.output_buffer.dimensions.height as u32,
    );
    png_encoder.set_depth(png::BitDepth::Eight);
    png_encoder.set_color(png::ColorType::Rgba);
    let mut png_writer = png_encoder
        .write_header()
        .unwrap()
        .into_stream_writer_with_size(state.output_buffer.dimensions.unpadded_bytes_per_row)
        .unwrap();

    // chunky
    info!("Next: Encode {}us", (Instant::now() - start).as_micros());
    for chunk in padded_buffer.chunks(state.output_buffer.dimensions.padded_bytes_per_row) {
        png_writer
            .write_all(&chunk[..state.output_buffer.dimensions.unpadded_bytes_per_row])
            .unwrap();
    }

    info!("Done {}us", (Instant::now() - start).as_micros());

    Ok(())
}

impl State {
    async fn new() -> Result<Self> {
        // TODO: Better handled in other example
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        // let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Pixel Size Uniform"),
        //     contents: bytemuck::cast_slice(&[1. / size.width as f32, 1. / size.height as f32]),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        // });

        // let surface_caps = surface.get_capabilities(&adapter);

        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .copied()
        //     .find(|f| f.is_srgb())
        //     .unwrap_or(surface_caps.formats[0]);

        // let config = wgpu::SurfaceConfiguration {
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        //     format: surface_format,
        //     width: size.width,
        //     height: size.height,
        //     present_mode: wgpu::PresentMode::Immediate,
        //     alpha_mode: surface_caps.alpha_modes[0],
        //     view_formats: vec![],
        // };
        // surface.configure(&device, &config);

        // let diffuse_bytes = include_bytes!("../../../../../images/animal.png");
        // let diffuse_bytes = include_bytes!("../../../../../images/camera.jpg");
        // let diffuse_bytes = include_bytes!("../../../../../images/gecko.jpg");
        // let diffuse_texture =
        //     DiffuseTexture::from_bytes(&device, &queue, diffuse_bytes, "Gecko").unwrap();

        // let img_bytes = include_bytes!("../../../../../images/camera.jpg");
        let img_bytes = include_bytes!("../../../../../images/gecko.jpg");
        let img = image::load_from_memory(img_bytes)?;

        let (diffuse_texture, render_texture, output_buffer) =
            texture::prepare(&device, &queue, &img)?;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 0,
                    //     visibility: wgpu::ShaderStages::FRAGMENT,
                    //     ty: wgpu::BindingType::Buffer {
                    //         ty: wgpu::BufferBindingType::Uniform,
                    //         has_dynamic_offset: false,
                    //         min_binding_size: None,
                    //     },
                    //     count: None,
                    // },
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
                // wgpu::BindGroupEntry {
                //     binding: 0,
                //     resource: uniform_buf.as_entire_binding(),
                // },
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
                entry_point: "fullscreen_vertex_shader",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "frag_shader",
                targets: &[Some(wgpu::ColorTargetState {
                    // TODO: Consolidate these
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // TODO: Remove, debugging
        // let render_timestamp = Instant::now();

        Ok(Self {
            device,
            queue,
            render_pipeline,
            diffuse_texture,
            diffuse_bind_group,
            // render_timestamp,
            output_buffer,
            render_texture,
        })
    }

    // pub fn window(&self) -> &Window {
    //     &self.window
    // }

    // pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    //     if new_size.width > 0 && new_size.height > 0 {
    //         self.size = new_size;
    //         self.config.width = new_size.width;
    //         self.config.height = new_size.height;
    //         self.surface.configure(&self.device, &self.config);
    //     }
    // }

    // #[allow(unused_variables)]
    // fn input(&mut self, event: &WindowEvent) -> bool {
    //     false
    // }

    // fn update(&mut self) {
    //     let now = Instant::now();
    //     let previous = self.render_timestamp;
    //     self.render_timestamp = now;

    //     let diff = now - previous;

    //     info!("t {}", diff.as_micros());
    // }

    fn render(&mut self) -> std::result::Result<wgpu::SubmissionIndex, wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // TODO: Is this the correct order of operations?
        // We complete the render pass, then we do the copy texture?
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

// pub async fn run() -> Result<()> {
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();

//     // State::new uses async code, so we're going to wait for it to finish
//     let mut state = State::new(window).await;

//     event_loop.run(move |event, _, control_flow| {
//         control_flow.set_poll();

//         match event {
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == state.window().id() => {
//                 if !state.input(event) {
//                     match event {
//                         WindowEvent::CloseRequested
//                         | WindowEvent::KeyboardInput {
//                             input:
//                                 KeyboardInput {
//                                     state: ElementState::Pressed,
//                                     virtual_keycode: Some(VirtualKeyCode::Escape),
//                                     ..
//                                 },
//                             ..
//                         } => *control_flow = ControlFlow::Exit,
//                         WindowEvent::Resized(physical_size) => {
//                             state.resize(*physical_size);
//                         }
//                         WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                             // new_inner_size is &mut so w have to dereference it twice
//                             state.resize(**new_inner_size);
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//             Event::RedrawRequested(window_id) if window_id == state.window().id() => {
//                 state.update();
//                 match state.render() {
//                     Ok(_) => {}
//                     // Reconfigure the surface if it's lost or outdated
//                     Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
//                         state.resize(state.size)
//                     }
//                     // The system is out of memory, we should probably quit
//                     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
//                     // We're ignoring timeouts
//                     Err(wgpu::SurfaceError::Timeout) => warn!("Surface timeout"),
//                 }
//             }
//             Event::MainEventsCleared => {
//                 // RedrawRequested will only trigger once, unless we manually
//                 // request it.
//                 state.window().request_redraw();
//             }
//             _ => {}
//         }
//     });
// }
