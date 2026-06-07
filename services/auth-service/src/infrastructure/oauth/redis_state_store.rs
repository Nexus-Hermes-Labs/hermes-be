use async_trait::async_trait;
use redis::aio::ConnectionManager;

use crate::application::ports::oauth_state_store::{
    OAuthFlowState, OAuthStateStore, OAuthStateStoreError,
};

const KEY_PREFIX: &str = "oauth:state:";

/// Redis-backed [`OAuthStateStore`].
///
/// Stores the PKCE `code_verifier` under `oauth:state:{state}` with a TTL and
/// consumes it atomically with `GETDEL` (single-use), so a leaked or replayed
/// `state` cannot be exchanged twice.
#[derive(Clone)]
pub struct RedisOAuthStateStore {
    redis: ConnectionManager,
}

impl RedisOAuthStateStore {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    fn key(state: &str) -> String {
        format!("{KEY_PREFIX}{state}")
    }
}

#[async_trait]
impl OAuthStateStore for RedisOAuthStateStore {
    async fn save(
        &self,
        state: &str,
        flow: &OAuthFlowState,
        ttl_seconds: u64,
    ) -> Result<(), OAuthStateStoreError> {
        let mut conn = self.redis.clone();
        redis::cmd("SET")
            .arg(Self::key(state))
            .arg(&flow.code_verifier)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| OAuthStateStoreError::Backend(e.to_string()))
    }

    async fn take(&self, state: &str) -> Result<Option<OAuthFlowState>, OAuthStateStoreError> {
        let mut conn = self.redis.clone();
        let code_verifier: Option<String> = redis::cmd("GETDEL")
            .arg(Self::key(state))
            .query_async(&mut conn)
            .await
            .map_err(|e| OAuthStateStoreError::Backend(e.to_string()))?;

        Ok(code_verifier.map(|code_verifier| OAuthFlowState { code_verifier }))
    }
}
