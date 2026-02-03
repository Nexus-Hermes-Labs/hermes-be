// Common library for Hermes
// Shared types, utilities, and infrastructure code

pub mod config;
pub mod db;
pub mod error;
pub mod events;
pub mod jwt;
pub mod message_queue;
pub mod middleware;
pub mod observability;
pub mod pagination;
pub mod persistance;
pub mod utils;
pub mod dto;

// Re-export commonly used types
pub use error::{AppError, Result};
pub use events::Event;
pub use persistance::repository::Repository;
