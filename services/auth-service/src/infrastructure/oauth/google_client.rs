use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use common_config::oauth::GoogleOAuthConfig;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::application::ports::oauth_provider::{
    AuthorizeRedirect, GoogleOAuthClient, OAuthProviderError, OAuthUserInfo,
};

const GOOGLE_SCOPES: &str = "openid email profile";

/// `reqwest`-backed [`GoogleOAuthClient`].
///
/// Holds the provider config as `Option`: when Google is not configured every
/// method short-circuits with [`OAuthProviderError::NotConfigured`], so the
/// service can be wired unconditionally and report 503 at request time.
pub struct ReqwestGoogleClient {
    config: Option<GoogleOAuthConfig>,
    http: Client,
}

impl ReqwestGoogleClient {
    pub fn new(config: Option<GoogleOAuthConfig>) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    fn cfg(&self) -> Result<&GoogleOAuthConfig, OAuthProviderError> {
        self.config.as_ref().ok_or(OAuthProviderError::NotConfigured)
    }
}

#[async_trait]
impl GoogleOAuthClient for ReqwestGoogleClient {
    fn build_authorize_url(&self, state: &str) -> Result<AuthorizeRedirect, OAuthProviderError> {
        let cfg = self.cfg()?;

        // PKCE: random verifier (alphanumeric is a valid `unreserved` subset),
        // S256 challenge = base64url(sha256(verifier)) without padding.
        let code_verifier: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        let code_challenge =
            URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));

        let url = reqwest::Url::parse_with_params(
            &cfg.auth_url,
            &[
                ("response_type", "code"),
                ("client_id", cfg.client_id.as_str()),
                ("redirect_uri", cfg.redirect_uri.as_str()),
                ("scope", GOOGLE_SCOPES),
                ("state", state),
                ("code_challenge", code_challenge.as_str()),
                ("code_challenge_method", "S256"),
            ],
        )
        .map_err(|e| OAuthProviderError::Request(e.to_string()))?;

        Ok(AuthorizeRedirect {
            url: url.to_string(),
            code_verifier,
        })
    }

    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<String, OAuthProviderError> {
        let cfg = self.cfg()?;

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let response = self
            .http
            .post(&cfg.token_url)
            .form(&[
                ("code", code),
                ("client_id", cfg.client_id.as_str()),
                ("client_secret", cfg.client_secret.as_str()),
                ("redirect_uri", cfg.redirect_uri.as_str()),
                ("grant_type", "authorization_code"),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
            .map_err(|e| OAuthProviderError::Request(e.to_string()))?
            .error_for_status()
            .map_err(|e| OAuthProviderError::Request(e.to_string()))?;

        let token: TokenResponse = response
            .json()
            .await
            .map_err(|e| OAuthProviderError::InvalidResponse(e.to_string()))?;

        Ok(token.access_token)
    }

    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthProviderError> {
        let cfg = self.cfg()?;

        #[derive(Deserialize)]
        struct GoogleProfile {
            sub: String,
            email: String,
            #[serde(default)]
            email_verified: bool,
            #[serde(default)]
            name: Option<String>,
        }

        let response = self
            .http
            .get(&cfg.userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OAuthProviderError::Request(e.to_string()))?
            .error_for_status()
            .map_err(|e| OAuthProviderError::Request(e.to_string()))?;

        let profile: GoogleProfile = response
            .json()
            .await
            .map_err(|e| OAuthProviderError::InvalidResponse(e.to_string()))?;

        Ok(OAuthUserInfo {
            provider_user_id: profile.sub,
            email: profile.email,
            email_verified: profile.email_verified,
            display_name: profile.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> GoogleOAuthConfig {
        GoogleOAuthConfig {
            client_id: "client-123".to_string(),
            client_secret: "secret-xyz".to_string(),
            redirect_uri: "http://localhost:3001/oauth/google/callback".to_string(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
        }
    }

    #[test]
    fn authorize_url_contains_pkce_and_state() {
        let client = ReqwestGoogleClient::new(Some(test_config()));
        let redirect = client.build_authorize_url("state-abc").unwrap();

        assert!(redirect.url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
        assert!(redirect.url.contains("state=state-abc"));
        assert!(redirect.url.contains("code_challenge_method=S256"));
        assert!(redirect.url.contains("code_challenge="));
        assert!(redirect.url.contains("client_id=client-123"));
        // Verifier is 64 chars and never embedded in the URL.
        assert_eq!(redirect.code_verifier.len(), 64);
        assert!(!redirect.url.contains(&redirect.code_verifier));
    }

    #[test]
    fn unconfigured_client_reports_not_configured() {
        let client = ReqwestGoogleClient::new(None);
        let err = client.build_authorize_url("state").unwrap_err();
        assert!(matches!(err, OAuthProviderError::NotConfigured));
    }
}
