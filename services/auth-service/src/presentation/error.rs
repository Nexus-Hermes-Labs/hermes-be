use thiserror::Error;

#[derive(Debug, Error)]
pub enum PresentationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Addr parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("HTTP server error: {0}")]
    HttpServer(String),

    #[error("gRPC server error: {0}")]
    GrpcServer(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
