use thiserror::Error;

/// Re-export of [`std::result::Result`] but using our own [`enum@Error`].
/// All fallible operations in this library should use this.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that may occur in this library.
#[derive(Debug, Error)]
pub enum Error {
    /// GPU related error.
    #[error("GPU error: {0}")]
    Gpu(String),

    /// IO transparent error.
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Image crate transparent error.
    #[error("Image library error: {0}")]
    Image(#[from] image::ImageError),
}
