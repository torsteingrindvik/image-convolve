use clap::ValueEnum;

/// The various backends available, enumerated.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Backend {
    /// See [`backends::cpu_single`].
    SingleNestedLoops,

    /// See [`backends::cpu_single`].
    SingleNestedIterators,

    /// See [`backends::cpu_multi`].
    MultiRayon,
}

/// Implementors of the [`Strategy`]
pub mod backends {
    /// Single threaded CPU convolution.
    pub mod cpu_single;

    /// Multi threaded CPU convolution.
    pub mod cpu_multi;

    /// Common CPU operations.
    pub(crate) mod cpu_util;
}

/// Holds the common trait for backends,
/// as well as the strategy implementation.
pub mod strategy;

/// Utilities for helping convolution.
pub(crate) mod util;
