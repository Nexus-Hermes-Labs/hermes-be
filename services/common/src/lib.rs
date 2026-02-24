// Common library for Hermes
// Shared types, utilities, and infrastructure code

pub mod db;
pub mod domain;
pub mod dto;
pub mod events;
pub mod infrastructure;
pub mod middleware;
pub mod observability;
pub mod pagination;
pub mod utils;

#[cfg(feature = "testing")]
pub mod test_utils;

// Re-export commonly used types
pub use events::Event;
pub use infrastructure::security::jwt_manager::SystemRole;
