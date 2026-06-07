use async_trait::async_trait;
use thiserror::Error;

/// Profile data returned by an OAuth provider's userinfo endpoint.
#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    /// Provider-stable subject id (Google `sub`). Never changes for a user.
    pub provider_user_id: String,
    /// Email reported by the provider.
    pub email: String,
    /// Whether the provider considers the email verified.
    pub email_verified: bool,
    /// Human-readable name, when the provider exposes one.
    pub display_name: Option<String>,
}

/// Result of starting an authorization request.
///
/// The provider client generates the PKCE `code_verifier` and embeds the
/// derived challenge in `url`. The application stores `code_verifier` against
/// the CSRF state so it can be replayed at the token-exchange step; it is never
/// sent to the browser.
#[derive(Debug, Clone)]
pub struct AuthorizeRedirect {
    pub url: String,
    pub code_verifier: String,
}

/// Errors raised by an OAuth provider client (network/transport/decoding,
/// or the provider simply not being configured).
#[derive(Debug, Error)]
pub enum OAuthProviderError {
    #[error("OAuth provider is not configured")]
    NotConfigured,

    #[error("OAuth provider request failed: {0}")]
    Request(String),

    #[error("OAuth provider returned an invalid response: {0}")]
    InvalidResponse(String),
}

/// Outbound port for the Google OAuth provider.
///
/// Implemented in the infrastructure layer (`ReqwestGoogleClient`). Modelled as
/// a trait so tests can inject a deterministic fake instead of hitting Google.
#[async_trait]
pub trait GoogleOAuthClient: Send + Sync {
    /// Build the Google consent URL for a CSRF `state`, generating a fresh PKCE
    /// pair. Returns the URL plus the `code_verifier` the caller must persist.
    fn build_authorize_url(&self, state: &str) -> Result<AuthorizeRedirect, OAuthProviderError>;

    /// Exchange an authorization `code` (with its PKCE `code_verifier`) for a
    /// Google access token.
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<String, OAuthProviderError>;

    /// Fetch the authenticated user's profile using a Google access token.
    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthProviderError>;
}
