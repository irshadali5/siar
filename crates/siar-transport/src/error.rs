use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("failed to bind endpoint: {0}")]
    Bind(String),
    #[error("connection failed: {0}")]
    Connect(String),
    #[error("stream write failed: {0}")]
    Write(String),
    #[error("stream read failed: {0}")]
    Read(String),
    #[error(transparent)]
    Codec(#[from] siar_protocol::CodecError),
}
