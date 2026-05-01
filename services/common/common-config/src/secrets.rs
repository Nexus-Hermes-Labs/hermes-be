// config/secrets.rs
use serde::Deserialize;
use std::time::Duration;

/// Secrets configuration
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SecretsConfig {
    pub jwt: JwtSecrets,
    pub password: PasswordSecrets,
}

/// JWT-related secrets
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct JwtSecrets {
    #[serde(default = "default_jwt_issuer")]
    pub issuer: String,
    #[serde(default = "default_jwt_access_secret")]
    pub access_secret: String,
    #[serde(default = "default_jwt_refresh_secret")]
    pub refresh_secret: String,
    #[serde(default = "default_jwt_expiration")]
    pub expiration_hours: i64,
    #[serde(default = "default_jwt_refresh_expiration")]
    pub refresh_expiration_days: i64,
}

fn default_jwt_access_secret() -> String {
    "default_access_secret_32_chars_long_placeholder".to_string()
}

fn default_jwt_refresh_secret() -> String {
    "default_refresh_secret_32_chars_long_placeholder".to_string()
}

fn default_jwt_issuer() -> String {
    "hermes".to_string()
}

fn default_jwt_expiration() -> i64 {
    24 // 24 hours
}

fn default_jwt_refresh_expiration() -> i64 {
    30 // 30 days
}

impl Default for JwtSecrets {
    fn default() -> Self {
        Self {
            issuer: default_jwt_issuer(),
            access_secret: default_jwt_access_secret(),
            refresh_secret: default_jwt_refresh_secret(),
            expiration_hours: default_jwt_expiration(),
            refresh_expiration_days: default_jwt_refresh_expiration(),
        }
    }
}

impl JwtSecrets {
    /// Get access token expiration as Duration
    pub fn access_token_expiration(&self) -> Duration {
        Duration::from_secs((self.expiration_hours * 3600) as u64)
    }

    /// Get refresh token expiration as Duration
    pub fn refresh_token_expiration(&self) -> Duration {
        Duration::from_secs((self.refresh_expiration_days * 86400) as u64)
    }

    /// Validate JWT secrets
    pub fn validate(&self) -> Result<(), String> {
        // Skip validation for placeholder defaults in development
        if self.access_secret.contains("placeholder") {
            return Ok(());
        }

        if self.access_secret.len() < 32 {
            return Err("JWT access secret must be at least 32 characters".into());
        }

        if self.refresh_secret.len() < 32 {
            return Err("JWT refresh secret must be at least 32 characters".into());
        }

        if self.expiration_hours <= 0 {
            return Err("JWT expiration hours must be positive".into());
        }

        if self.refresh_expiration_days <= 0 {
            return Err("JWT refresh expiration days must be positive".into());
        }

        Ok(())
    }
}

/// Password-related secrets
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PasswordSecrets {
    #[serde(default = "default_password_pepper")]
    pub pepper: String,
    #[serde(default = "default_min_password_length")]
    pub min_length: usize,
    #[serde(default = "default_bcrypt_cost")]
    pub bcrypt_cost: u32,
}

fn default_password_pepper() -> String {
    "default_pepper_min_16_chars_placeholder".to_string()
}

fn default_min_password_length() -> usize {
    8
}

fn default_bcrypt_cost() -> u32 {
    12
}

impl Default for PasswordSecrets {
    fn default() -> Self {
        Self {
            pepper: default_password_pepper(),
            min_length: default_min_password_length(),
            bcrypt_cost: default_bcrypt_cost(),
        }
    }
}

impl PasswordSecrets {
    /// Validate password secrets
    pub fn validate(&self) -> Result<(), String> {
        // Skip validation for placeholder defaults in development
        if self.pepper.contains("placeholder") {
            return Ok(());
        }

        if self.pepper.len() < 16 {
            return Err("Password pepper must be at least 16 characters".into());
        }

        if self.min_length < 6 {
            return Err("Minimum password length must be at least 6".into());
        }

        if !(4..=31).contains(&self.bcrypt_cost) {
            return Err("Bcrypt cost must be between 4 and 31".into());
        }

        Ok(())
    }
}
