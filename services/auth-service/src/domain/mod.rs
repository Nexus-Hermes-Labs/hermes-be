pub mod auth_credential;
pub mod auth_session;
pub mod oauth_account;
pub mod password_history;
pub mod password_policy;
pub mod rate_limit;
pub mod unit_of_work;

// Re-export main types for convenience
pub use auth_session::{AuthSession, AuthSessionError, AuthSessionRepository};
pub use oauth_account::{OAuthAccount, OAuthAccountError, OAuthAccountRepository, OAuthProvider};
pub use password_history::{PasswordHistory, PasswordHistoryRepository};
pub use password_policy::PasswordPolicy;
pub use rate_limit::RateLimiter;
pub use unit_of_work::{CredentialWriter, OAuthAccountWriter, SessionWriter};
