use std::fmt;
use std::str::FromStr;

use super::error::OAuthAccountError;

// ============================================
// OAUTH PROVIDER ENUM
// ============================================

/// Supported social-login providers.
///
/// Only Google is implemented today; the enum is the seam where additional
/// providers (GitHub, etc.) plug in without touching the rest of the flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
}

impl OAuthProvider {
    /// Stable database/url slug for this provider.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Google => "google",
        }
    }
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for OAuthProvider {
    type Err = OAuthAccountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(Self::Google),
            other => Err(OAuthAccountError::UnknownProvider(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_roundtrip() {
        assert_eq!("google".parse::<OAuthProvider>().unwrap(), OAuthProvider::Google);
        assert_eq!(OAuthProvider::Google.as_str(), "google");
        assert_eq!(OAuthProvider::Google.to_string(), "google");
    }

    #[test]
    fn test_unknown_provider() {
        assert!("facebook".parse::<OAuthProvider>().is_err());
    }
}
