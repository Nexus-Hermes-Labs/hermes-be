use thiserror::Error;

/// Presentation layer errors (server startup failures)
#[derive(Debug, Error)]
pub enum PresentationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("HTTP server error: {0}")]
    HttpServer(String),

    #[error("gRPC server error: {0}")]
    GrpcServer(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
