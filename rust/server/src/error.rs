use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}
