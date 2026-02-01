use crate::domain::user::valueobject::PasswordHashVO;

pub trait PasswordService: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Plain text to hashed-password
    fn hash_password(&self, plain: &str) -> Result<PasswordHashVO, Self::Error>;

    /// Verify hashed password
    fn verify_password(&self, plain: &str, hash: &PasswordHashVO) -> Result<bool, Self::Error>;
}
