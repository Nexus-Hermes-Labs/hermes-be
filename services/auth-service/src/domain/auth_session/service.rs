use crate::domain::auth_session::error::AuthSessionError;
use crate::domain::auth_session::valueobject::RefreshTokenHash;

pub trait TokenHasher: Send + Sync {
    fn hash(&self, token: &str) -> Result<RefreshTokenHash, AuthSessionError>;
}
