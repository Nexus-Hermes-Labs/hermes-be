// Common library for Hermes
// Shared types, utilities, and infrastructure code

pub mod config;
pub mod db;
pub mod domain;
pub mod dto;
pub mod events;
pub mod infrastructure;
pub mod middleware;
pub mod observability;
pub mod pagination;
pub mod proto;
pub mod utils;

// Re-export commonly used types
pub use events::Event;
