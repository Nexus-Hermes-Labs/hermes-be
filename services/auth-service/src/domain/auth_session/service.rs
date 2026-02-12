use async_trait::async_trait;
use crate::domain::auth_session::valueobject::RefreshTokenHash;

#[async_trait]
pub trait TokenHasher {
    type Error;

    fn hash(&self, token: &str) -> Result<RefreshTokenHash, Self::Error>;
}