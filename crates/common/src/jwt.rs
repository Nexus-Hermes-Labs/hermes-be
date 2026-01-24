use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid, // User ID
    pub email: String,
    pub role: String,
    pub jti: Uuid,
    pub typ: TokenType,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Clone)]
pub struct JwtManager {
    access_encoding_key: EncodingKey,
    access_decoding_key: DecodingKey,
    refresh_encoding_key: EncodingKey,
    refresh_decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    pub fn new(access_secret: &str, refresh_secret: &str) -> Self {
        Self {
            access_encoding_key: EncodingKey::from_secret(access_secret.as_bytes()),
            access_decoding_key: DecodingKey::from_secret(access_secret.as_bytes()),
            refresh_encoding_key: EncodingKey::from_secret(refresh_secret.as_bytes()),
            refresh_decoding_key: DecodingKey::from_secret(refresh_secret.as_bytes()),
            validation: Validation::default(),
        }
    }

    /// Create an access token (short-lived, for API access)
    pub fn create_user_token(
        &self,
        user_id: Uuid,
        email: String,
        role: String,
        expiration_hours: i64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = chrono::Utc::now();
        let exp = (now + chrono::Duration::hours(expiration_hours)).timestamp();

        let claims = Claims {
            sub: user_id,
            email,
            role,
            jti: Uuid::new_v4(),
            typ: TokenType::Access,
            exp,
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.access_encoding_key)
    }

    /// Create a refresh token (long-lived, only for refreshing)
    pub fn create_refresh_token(
        &self,
        user_id: Uuid,
        email: String,
        role: String,
        expiration_days: i64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = chrono::Utc::now();
        let exp = (now + chrono::Duration::days(expiration_days)).timestamp();

        let claims = Claims {
            sub: user_id,
            email,
            role,
            jti: Uuid::new_v4(),
            typ: TokenType::Refresh,
            exp,
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.refresh_encoding_key)
    }

    /// Verify access token (used in middleware)
    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(token, &self.access_decoding_key, &self.validation)?;

        let claims = token_data.claims;

        // ⭐ Check token type
        if claims.typ != TokenType::Access {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        Ok(claims)
    }

    /// Verify refresh token (only for refresh endpoint)
    pub fn verify_refresh_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(token, &self.refresh_decoding_key, &self.validation)?;

        let claims = token_data.claims;

        // ⭐ Check token type
        if claims.typ != TokenType::Refresh {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        Ok(claims)
    }
}

//TODO: Token Blacklist for token thieves that try to reuse old tokens!
