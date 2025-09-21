//! Error types for iroh-mesh

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Iroh error: {0}")]
    Iroh(#[from] iroh::endpoint::ConnectionError),

    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Proxy error: {0}")]
    Proxy(String),

    #[error("Agent error: {0}")]
    Agent(String),
}