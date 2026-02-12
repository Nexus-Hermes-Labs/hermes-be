use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("Failed to connect to message broker: {0}")]
    ConnectionFailed(String),

    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Failed to serialize event: {0}")]
    SerializationFailed(String),

    #[error("Failed to deserialize event: {0}")]
    DeserializationFailed(String),

    #[error("Timeout waiting for acknowledgment")]
    Timeout,
}
