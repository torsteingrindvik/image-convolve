use clap::ValueEnum;

/// The various backends available, enumerated.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Backend {
    /// See [`backends::cpu::single`].
    SingleNestedLoops,

    /// See [`backends::cpu::single`].
    SingleNestedIterators,

    /// See [`backends::cpu::multi`].
    MultiRayon,

    /// See [`backends::gpu::offscreen`].
    GpuOffscreen,
}

/// Implementors of the [`strategy::ConvolveStrategy`]
pub mod backends {
    /// CPU based convolution.
    pub mod cpu {
        /// Single threaded.
        pub mod single;

        /// Multi threaded.
        pub mod multi;

        /// Common CPU operations.
        pub(crate) mod util;
    }

    /// GPU based convolution.
    pub mod gpu {
        /// Convolution via an offscreen GPU pipeline.
        pub mod offscreen;
    }
}

/// Holds the common trait for backends,
/// as well as the strategy implementation.
pub mod strategy;
