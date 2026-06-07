// common-config/src/oauth.rs
use serde::Deserialize;

/// OAuth / social-login configuration.
///
/// Optional at the top level so services that don't use OAuth (everything
/// except `auth-service`) are not forced to provide credentials. Each provider
/// is itself optional; when a provider is absent the corresponding login flow
/// reports `ProviderNotConfigured` at request time rather than failing at boot.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OAuthConfig {
    #[serde(default)]
    pub google: Option<GoogleOAuthConfig>,
}

/// Google OAuth 2.0 credentials and endpoints.
///
/// `client_id` / `client_secret` come from the Google Cloud console. The
/// `redirect_uri` points at the **frontend** callback route (Architecture A):
/// Google redirects the browser there, the SPA then POSTs `{code, state}` to
/// `auth-service`. The three endpoint URLs default to Google's public values
/// and rarely need overriding (useful for tests / future GCP regions).
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,

    #[serde(default = "default_google_auth_url")]
    pub auth_url: String,
    #[serde(default = "default_google_token_url")]
    pub token_url: String,
    #[serde(default = "default_google_userinfo_url")]
    pub userinfo_url: String,
}

fn default_google_auth_url() -> String {
    "https://accounts.google.com/o/oauth2/v2/auth".to_string()
}

fn default_google_token_url() -> String {
    "https://oauth2.googleapis.com/token".to_string()
}

fn default_google_userinfo_url() -> String {
    "https://www.googleapis.com/oauth2/v3/userinfo".to_string()
}

impl GoogleOAuthConfig {
    /// Validate Google OAuth config when it is present.
    pub fn validate(&self) -> Result<(), String> {
        if self.client_id.trim().is_empty() {
            return Err("Google OAuth client_id cannot be empty".into());
        }
        if self.client_secret.trim().is_empty() {
            return Err("Google OAuth client_secret cannot be empty".into());
        }
        if self.redirect_uri.trim().is_empty() {
            return Err("Google OAuth redirect_uri cannot be empty".into());
        }
        Ok(())
    }
}

impl OAuthConfig {
    /// Validate any configured providers. Absent providers are a no-op.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(google) = &self.google {
            google.validate()?;
        }
        Ok(())
    }
}
