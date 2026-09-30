use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse HTTP request: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
