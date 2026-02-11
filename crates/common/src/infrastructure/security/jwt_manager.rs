use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ============================================
// JWT CLAIMS (RFC 7519 Standard)
// ============================================

/// Standard JWT claims following RFC 7519
///
/// See: https://datatracker.ietf.org/doc/html/rfc7519#section-4.1
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    // ============================================
    // STANDARD CLAIMS (RFC 7519)
    // ============================================
    /// Issuer - Who created and signed this token
    /// Example: "auth-service" or "https://auth.hermes.com"
    pub iss: String,

    /// Subject - Whom the token refers to (user ID)
    /// Standard says this should be a string, so we convert Uuid to string
    pub sub: String,

    /// Audience - Who the token is intended for
    /// Example: "api-service", "user-service"
    /// Can be a single string or array, we use single for simplicity
    pub aud: String,

    /// Expiration Time - When the token expires (Unix timestamp)
    pub exp: i64,

    /// Not Before - Time before which token must not be accepted (Unix timestamp)
    pub nbf: i64,

    /// Issued At - When the token was issued (Unix timestamp)
    pub iat: i64,

    /// JWT ID - Unique identifier for this token
    /// Can be used to prevent replay attacks or for token revocation
    pub jti: String,

    // ============================================
    // CUSTOM CLAIMS (Application-specific)
    // ============================================
    /// Token type: "access" or "refresh"
    pub typ: TokenType,

    /// User's email (for convenience)
    pub email: String,
}

impl Claims {
    /// Get user ID from subject claim
    pub fn user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }

    /// Get JWT ID as Uuid
    pub fn jwt_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.jti)
    }

    /// Check if token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() >= self.exp
    }

    /// Check if token is valid yet (nbf check)
    pub fn is_valid_yet(&self) -> bool {
        Utc::now().timestamp() >= self.nbf
    }

    /// Check if token is valid (not expired and valid already)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && self.is_valid_yet()
    }
}

// ============================================
// TOKEN TYPE
// ============================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

// ============================================
// JWT MANAGER
// ============================================

/// JWT token manager for creating and verifying tokens
///
/// Uses separate secrets for access and refresh tokens for enhanced security.
/// If one secret is compromised, the other token type remains secure.
#[derive(Clone)]
pub struct JwtManager {
    /// Issuer name (e.g., "auth-service")
    issuer: String,

    /// Audience for access tokens (e.g., "api")
    access_audience: String,

    /// Audience for refresh tokens (e.g., "auth")
    refresh_audience: String,

    /// Encoding key for access tokens
    access_encoding_key: EncodingKey,

    /// Decoding key for access tokens
    access_decoding_key: DecodingKey,

    /// Encoding key for refresh tokens
    refresh_encoding_key: EncodingKey,

    /// Decoding key for refresh tokens
    refresh_decoding_key: DecodingKey,

    /// Validation settings
    validation: Validation,
}

impl JwtManager {
    /// Create a new JWT manager
    ///
    /// # Arguments
    /// * `issuer` - Issuer name (e.g., "auth-service")
    /// * `access_secret` - Secret for access tokens (min 32 bytes recommended)
    /// * `refresh_secret` - Secret for refresh tokens (min 32 bytes recommended)
    ///
    /// # Security Notes
    /// - Use different secrets for access and refresh tokens
    /// - Secrets should be cryptographically random
    /// - Secrets should be at least 32 bytes (256 bits)
    /// - Store secrets in environment variables or secret manager
    pub fn new(issuer: &str, access_secret: &str, refresh_secret: &str) -> Result<Self, JwtError> {
        // Validate secret lengths
        if access_secret.len() < 32 {
            return Err(JwtError::ConfigurationError(
                "Access secret must be at least 32 characters".to_string(),
            ));
        }

        if refresh_secret.len() < 32 {
            return Err(JwtError::ConfigurationError(
                "Refresh secret must be at least 32 characters".to_string(),
            ));
        }

        let mut validation = Validation::default();
        validation.validate_exp = true; // Validate expiration
        validation.validate_nbf = true; // Validate not-before
        validation.set_required_spec_claims(&["iss", "sub", "aud", "exp", "iat", "jti"]);

        Ok(Self {
            issuer: issuer.into(),
            access_audience: "api".to_string(),
            refresh_audience: "auth".to_string(),
            access_encoding_key: EncodingKey::from_secret(access_secret.as_bytes()),
            access_decoding_key: DecodingKey::from_secret(access_secret.as_bytes()),
            refresh_encoding_key: EncodingKey::from_secret(refresh_secret.as_bytes()),
            refresh_decoding_key: DecodingKey::from_secret(refresh_secret.as_bytes()),
            validation,
        })
    }

    /// Create an access token (short-lived, for API access)
    ///
    /// # Arguments
    /// * `user_id` - User's unique identifier
    /// * `email` - User's email address
    /// * `expiration_hours` - Token lifetime in hours (typically 6-24)
    pub fn create_access_token(
        &self,
        user_id: Uuid,
        email: impl Into<String>,
        expiration_hours: i64,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = (now + Duration::hours(expiration_hours)).timestamp();
        let nbf = now.timestamp(); // Valid immediately

        let claims = Claims {
            iss: self.issuer.clone(),
            sub: user_id.to_string(),
            aud: self.access_audience.clone(),
            exp,
            nbf,
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            typ: TokenType::Access,
            email: email.into(),
        };

        encode(&Header::default(), &claims, &self.access_encoding_key)
            .map_err(JwtError::EncodingError)
    }

    /// Create a refresh token (long-lived, only for refreshing)
    ///
    /// # Arguments
    /// * `user_id` - User's unique identifier
    /// * `email` - User's email address
    /// * `expiration_days` - Token lifetime in days (typically 7-30)
    pub fn create_refresh_token(
        &self,
        user_id: Uuid,
        email: impl Into<String>,
        expiration_days: i64,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = (now + Duration::days(expiration_days)).timestamp();
        let nbf = now.timestamp();

        let claims = Claims {
            iss: self.issuer.clone(),
            sub: user_id.to_string(),
            aud: self.refresh_audience.clone(),
            exp,
            nbf,
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            typ: TokenType::Refresh,
            email: email.into(),
        };

        encode(&Header::default(), &claims, &self.refresh_encoding_key)
            .map_err(JwtError::EncodingError)
    }

    /// Verify and decode an access token
    ///
    /// This validates:
    /// - Token signature
    /// - Token expiration (exp)
    /// - Token not-before (nbf)
    /// - Token type (must be "access")
    /// - Required claims presence
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = self.validation.clone();
        validation.set_audience(&[&self.access_audience]);
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<Claims>(token, &self.access_decoding_key, &validation)
            .map_err(JwtError::DecodingError)?;

        let claims = token_data.claims;

        // Verify token type
        if claims.typ != TokenType::Access {
            return Err(JwtError::InvalidTokenType {
                expected: TokenType::Access,
                found: claims.typ,
            });
        }

        Ok(claims)
    }

    /// Verify and decode a refresh token
    ///
    /// This validates:
    /// - Token signature
    /// - Token expiration (exp)
    /// - Token not-before (nbf)
    /// - Token type (must be "refresh")
    /// - Required claims presence
    pub fn verify_refresh_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = self.validation.clone();
        validation.set_audience(&[&self.refresh_audience]);
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<Claims>(token, &self.refresh_decoding_key, &validation)
            .map_err(JwtError::DecodingError)?;

        let claims = token_data.claims;

        // Verify token type
        if claims.typ != TokenType::Refresh {
            return Err(JwtError::InvalidTokenType {
                expected: TokenType::Refresh,
                found: claims.typ,
            });
        }

        Ok(claims)
    }

    /// Get issuer name
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
}

// ============================================
// ERRORS
// ============================================

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("JWT encoding error: {0}")]
    EncodingError(#[from] jsonwebtoken::errors::Error),

    #[error("JWT decoding error: {0}")]
    DecodingError(jsonwebtoken::errors::Error),

    #[error("Invalid token type: expected {expected:?}, found {found:?}")]
    InvalidTokenType {
        expected: TokenType,
        found: TokenType,
    },

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> JwtManager {
        JwtManager::new(
            "test-auth-service",
            "test_access_secret_minimum_32_chars_long",
            "test_refresh_secret_minimum_32_chars_long",
        )
        .unwrap()
    }

    #[test]
    fn test_create_and_verify_access_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let token = manager
            .create_access_token(user_id, "test@example.com", 1)
            .unwrap();

        let claims = manager.verify_access_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.typ, TokenType::Access);
        assert_eq!(claims.iss, "test-auth-service");
        assert_eq!(claims.aud, "api");
    }

    #[test]
    fn test_create_and_verify_refresh_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let token = manager
            .create_refresh_token(user_id, "test@example.com", 7)
            .unwrap();

        let claims = manager.verify_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.typ, TokenType::Refresh);
        assert_eq!(claims.aud, "auth");
    }

    #[test]
    fn test_cannot_use_refresh_token_as_access_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let refresh_token = manager
            .create_refresh_token(user_id, "test@example.com", 7)
            .unwrap();

        let result = manager.verify_access_token(&refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_use_access_token_as_refresh_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let access_token = manager
            .create_access_token(user_id, "test@example.com", 1)
            .unwrap();

        let result = manager.verify_refresh_token(&access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_helper_methods() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let token = manager
            .create_access_token(user_id, "test@example.com", 1)
            .unwrap();

        let claims = manager.verify_access_token(&token).unwrap();

        assert_eq!(claims.user_id().unwrap(), user_id);
        assert!(claims.jwt_id().is_ok());
        assert!(!claims.is_expired());
        assert!(claims.is_valid_yet());
        assert!(claims.is_valid());
    }

    #[test]
    fn test_weak_secret_rejected() {
        let result = JwtManager::new(
            "test",
            "short_secret",
            "test_refresh_secret_minimum_32_chars_long",
        );
        assert!(result.is_err());
    }
}
