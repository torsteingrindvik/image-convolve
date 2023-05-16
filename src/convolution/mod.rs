/// Implementors of the [`Strategy`]
pub mod backends {
    /// Convolution using a naive byte-for-byte approach on the CPU.
    pub mod cpu_naive;

    /// Convolution using rayon for using all cores available.
    pub mod cpu_rayon;
}

/// Holds the common trait for backends,
/// as well as the strategy implementation.
pub mod strategy;

/// Utilities for helping convolution.
pub(crate) mod util;
