pub mod grpc;
pub mod persistence;

pub use persistence::postgres::channel::repository::PostgresChannelRepository;
