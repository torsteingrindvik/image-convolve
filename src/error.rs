/// Re-export of [`std::result::Result`] but using our own [`Error`].
/// All fallible operations in this library should use this.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that may occur in this library.
pub enum Error {}
