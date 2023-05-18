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
        // Convolution via a offscreen GPU pipeline.
        // pub mod offscreen;
    }
}

/// Holds the common trait for backends,
/// as well as the strategy implementation.
pub mod strategy;
