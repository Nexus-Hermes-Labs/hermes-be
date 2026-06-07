use async_trait::async_trait;
use thiserror::Error;

/// Transient state for an in-flight OAuth authorization, keyed by the CSRF
/// `state` value. Holds the PKCE `code_verifier` until the callback exchanges
/// the code. Short-lived and single-use.
#[derive(Debug, Clone)]
pub struct OAuthFlowState {
    pub code_verifier: String,
}

#[derive(Debug, Error)]
pub enum OAuthStateStoreError {
    #[error("OAuth state store backend error: {0}")]
    Backend(String),
}

/// Outbound port for persisting OAuth flow state between the authorize and
/// callback steps. Backed by Redis in production (TTL-bounded, single-use).
#[async_trait]
pub trait OAuthStateStore: Send + Sync {
    /// Store `flow` under `state` with a time-to-live in seconds.
    async fn save(
        &self,
        state: &str,
        flow: &OAuthFlowState,
        ttl_seconds: u64,
    ) -> Result<(), OAuthStateStoreError>;

    /// Atomically fetch-and-delete the flow for `state`. Returns `None` if the
    /// state is unknown or already consumed/expired (single-use semantics).
    async fn take(&self, state: &str) -> Result<Option<OAuthFlowState>, OAuthStateStoreError>;
}
