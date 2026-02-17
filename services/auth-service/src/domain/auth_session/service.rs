use crate::domain::auth_session::valueobject::RefreshTokenHash;
use crate::domain::auth_session::error::AuthSessionError;

pub trait TokenHasher: Send + Sync {
    fn hash(&self, token: &str) -> Result<RefreshTokenHash, AuthSessionError>;
}