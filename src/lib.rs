#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

pub(crate) mod cli;

/// Kernels available.
pub mod kernel;

/// Re-exports most used parts of this lib for convenience.
pub mod prelude;

/// Errors that can happen within this lib.
pub mod error;

/// Image convolution.
pub mod convolution;
