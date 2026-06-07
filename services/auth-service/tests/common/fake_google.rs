use std::sync::Mutex;

use async_trait::async_trait;
use auth_service::application::ports::oauth_provider::{
    AuthorizeRedirect, GoogleOAuthClient, OAuthProviderError, OAuthUserInfo,
};

/// Deterministic in-memory [`GoogleOAuthClient`] for tests.
///
/// `build_authorize_url` echoes the `state` and returns a fixed verifier;
/// `exchange_code` returns a dummy token; `fetch_user_info` returns whatever
/// profile the test set via [`FakeGoogleClient::set_user_info`].
pub struct FakeGoogleClient {
    user_info: Mutex<OAuthUserInfo>,
}

impl FakeGoogleClient {
    pub fn new(user_info: OAuthUserInfo) -> Self {
        Self {
            user_info: Mutex::new(user_info),
        }
    }

    pub fn set_user_info(&self, user_info: OAuthUserInfo) {
        *self.user_info.lock().expect("fake google lock") = user_info;
    }
}

impl Default for FakeGoogleClient {
    fn default() -> Self {
        Self::new(OAuthUserInfo {
            provider_user_id: "fake-sub-default".to_string(),
            email: "fake-default@example.com".to_string(),
            email_verified: true,
            display_name: Some("Fake Default".to_string()),
        })
    }
}

#[async_trait]
impl GoogleOAuthClient for FakeGoogleClient {
    fn build_authorize_url(&self, state: &str) -> Result<AuthorizeRedirect, OAuthProviderError> {
        Ok(AuthorizeRedirect {
            url: format!("https://accounts.google.test/o/oauth2/v2/auth?state={state}"),
            code_verifier: "fake-code-verifier".to_string(),
        })
    }

    async fn exchange_code(
        &self,
        _code: &str,
        _code_verifier: &str,
    ) -> Result<String, OAuthProviderError> {
        Ok("fake-google-access-token".to_string())
    }

    async fn fetch_user_info(
        &self,
        _access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthProviderError> {
        Ok(self.user_info.lock().expect("fake google lock").clone())
    }
}
