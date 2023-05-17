# Image Convolve

## TL;DR 

### Installation

```norust
cargo install --path .
```

### What's available

```norust
image-convolve --help

Image convolution program. The input image will be convolved and saved to the given output path

Usage: image-convolve --input <INPUT> --output <OUTPUT> --kernel <KERNEL>

Options:
  -i, --input <INPUT>
          Path to input image

  -o, --output <OUTPUT>
          Path to output image

  -k, --kernel <KERNEL>
          Kernel to apply to image

          Possible values:
          - identity:
            The identity operation. Should leave the image as-is. TODO: A good test would be using this and hashing the in/out to see that we're unaffected
          - edge-detection1:
            Edge detection version 1
          - edge-detection2:
            Edge detection version 2
          - sharpen:
            Sharpening
          - box-blur:
            Box blur
          - gaussian-blur:
            Gaussian blur

  -h, --help
          Print help (see a summary with '-h')
```

### Typical use

```norust
image-convolve --input in.png --output out.png --kernel box-blur
```

## Goal

A command line interface which is able to: 

* Point to some image as input
* Apply a convolution matrix to it
* Store the result in an output image

Also, we want to be able to get a sense of performance using
different approaches.


## General project setup

In most project I always include (when applicable) a set of libraries:

* `tracing` and `tracing-subscriber` for logging (tracing) and consuming logs (traces)
* `clap` for the command line interface
* `thiserror` for error enumeration

_The list is longer, but these applied to this project._

## Considerations

### Edge handling

If a kernel would access a pixel outside the width / height of an image,
a constant color is used (black, value zero).

## Future

### Performance

* Checkout the [Rust Performance Book](https://nnethercote.github.io/perf-book/) for tips
  * Use `assert!(..)` to let the compiler optimize away bounds checks 
    * TODO: How do we prove our code contains bounds checks?
* Try employing [Flamegraph](https://github.com/jonhoo/inferno)s 


### Flexibility

* Allow changing the behaviour of edge handling