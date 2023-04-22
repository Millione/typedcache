#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Gets returned when a specific key couldn't be found.
    #[error("Key not found in cache")]
    KeyNotFound,
    /// Gets returned when a specific key couldn't be found and loading via the data-loader callback also failed.
    #[error("Key not found and could not be loaded into cache")]
    KeyNotFoundOrLoadable,
}
