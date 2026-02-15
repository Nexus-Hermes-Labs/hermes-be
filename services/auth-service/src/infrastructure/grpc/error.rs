use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserGrpcError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC call failed: {0}")]
    Status(#[from] tonic::Status),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}