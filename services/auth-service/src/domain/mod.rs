pub mod auth_credential;
pub mod auth_session;
pub mod unit_of_work;

// Re-export main types for convenience
pub use auth_session::{AuthSession, AuthSessionError, AuthSessionRepository};
pub use unit_of_work::{CredentialWriter, SessionWriter};
