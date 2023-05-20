# Image Convolve

![cat-original](./images/1280x720.jpg)
![cat-sharpen](./images/processed/1280x720-sharpen.jpg)
![cat-edge-detection2](./images/processed/1280x720-edge-detection2.jpg)


## TL;DR 

### Installation

```norust
cargo install --path .
```
### Usage

```norust
image-convolve --input images/1920x1080.jpg --output out.jpg --kernel sharpen --backend multi-rayon
```

### Benchmarks

Benchmarks are performed using [criterion](https://docs.rs/criterion/latest/criterion/).

```norust
cargo bench
```

### CLI `--help` output

```norust
image-convolve --help

Image convolution program. The input image will be convolved and saved to the given output path

Usage: image-convolve --input <INPUT> --output <OUTPUT> --kernel <KERNEL> --backend <BACKEND>

Options:
  -i, --input <INPUT>
          Path to input image

  -o, --output <OUTPUT>
          Path to output image

  -k, --kernel <KERNEL>
          Kernel to apply to image

          Possible values:
          - identity:        The identity operation
          - edge-detection1: Edge detection version 1
          - edge-detection2: Edge detection version 2
          - sharpen:         Sharpening
          - box-blur:        Box blur
          - gaussian-blur:   Gaussian blur

  -b, --backend <BACKEND>
          Backend to use for convolution

          Possible values:
          - single-nested-loops:     See [`backends::cpu::single`]
          - single-nested-iterators: See [`backends::cpu::single`]
          - multi-rayon:             See [`backends::cpu::multi`]
          - gpu-offscreen:           See [`backends::gpu::offscreen`]

  -h, --help
          Print help (see a summary with '-h')
```

## Benchmarking

To save a baseline for comparison, do:

```norust
cargo bench --bench images -- --save-baseline <name>
```

where `<name>` is some identifier.

To compare against this at some later point, do

```norust
cargo bench --bench images -- --baseline <name>
```

### What is actually benchmarked?

On CPU the time it takes to read the **prepared** input buffer and apply a convolution to it and move the resulting pixels into the **prepared** output buffer.

On GPU the time it takes to run a render pipeline on a bound **prepared** input texture and render it to a texture, then copy that texture to a buffer, map the buffer CPU side, then copying the pixels into a **prepared** output buffer.  

## Architecture

### External Libraries

* `tracing` and `tracing-subscriber` for logging (tracing) and consuming logs (traces)
* `clap` for the CLI
* `thiserror` for error enumeration
* `wgpu` for GPU use
* `rayon` for CPU parallel processing

### Overview

The program:

* Reads some input image
* Prepares it for convolution using some kernel
* Performs the convolution using a _backend_

The backends need only implement a common interface in order to be able to do the
convolution.

Current backends:

* CPU
  * Single threaded loop based pixel access
  * Single threaded iterator based pixel access
  * Multi threaded iterator based pixel access
* GPU
  * Offscreen render pipeline

## Limitations

### Edge handling

If a kernel would access a pixel outside the width / height of an image,
a constant color is used (black, value zero).
As of now, CPU backends always skip one row/column on each edge
GPU backends use a clamp to edge sampler

### Kernels

Only 3x3 pre-defined kernels are available right now.

## Future

### Performance

* Checkout the [Rust Performance Book](https://nnethercote.github.io/perf-book/) for tips
  * Use `assert!(..)` to let the compiler optimize away bounds checks 
    * In general, we could read the [bounds-check-cookbook](https://github.com/Shnatsel/bounds-check-cookbook/) to try to learn about when bounds might interfere
    * Alternatively, we could more agressively use `unsafe` and skip checks where appropriate.
* Try employing [Flamegraph](https://github.com/jonhoo/inferno)s 
* Try `wgpu::Features::TIMESTAMP_QUERY` for GPU more rendering timing, see [here](https://github.com/gfx-rs/wgpu/blob/3563849585ad6f3ea65b6c9be294e9190555eed3/wgpu/examples/mipmap/main.rs#LL203C9-L203C40)
* Try using [memmap](https://docs.rs/memmap/latest/memmap/struct.Mmap.html) if we care about fast file loading
* Try using separable kernels where possible

### Backends

* GPU compute
* GPU CUDA

## Attributions

This project borrows from several sources:

* For texture-buffer copies, see this [WGPU sample](https://github.com/gfx-rs/wgpu/blob/trunk/wgpu/examples/capture/main.rs)
* For the WGPU setup and texture loading, see [Learn WGPU part 5](https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/)
* For a fullscreen vertex shader, see [this Bevy shader](https://github.com/bevyengine/bevy/blob/main/crates/bevy_core_pipeline/src/fullscreen_vertex_shader/fullscreen.wgsl)

See the separate `README.md` in the `images` folder for where the images come from.
