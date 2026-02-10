#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshTokenHash(String);

impl RefreshTokenHash {
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
