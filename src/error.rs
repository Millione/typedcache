#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Key not found in cache")]
    KeyNotFound,
    #[error("Key not found and could not be loaded into cache")]
    KeyNotFoundOrLoadable,
}
