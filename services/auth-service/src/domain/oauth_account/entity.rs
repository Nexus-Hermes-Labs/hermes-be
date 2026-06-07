use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::valueobject::OAuthProvider;

/// OAuth Account Entity
///
/// Links a local [`AuthCredential`](crate::domain::auth_credential::AuthCredential)
/// to an external identity at a provider (e.g. Google). The pair
/// `(provider, provider_user_id)` is globally unique and maps to exactly one
/// credential.
#[derive(Debug, Clone)]
pub struct OAuthAccount {
    id: Uuid,
    credential_id: Uuid,
    provider: OAuthProvider,
    provider_user_id: String,
    email: String,
    created_at: DateTime<Utc>,
}

impl OAuthAccount {
    /// Create a new link (for first-time OAuth login or explicit linking).
    pub fn new(
        credential_id: Uuid,
        provider: OAuthProvider,
        provider_user_id: impl Into<String>,
        email: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            credential_id,
            provider,
            provider_user_id: provider_user_id.into(),
            email: email.into(),
            created_at: Utc::now(),
        }
    }

    /// Reconstruct from a database row.
    pub fn from_persisted(
        id: Uuid,
        credential_id: Uuid,
        provider: OAuthProvider,
        provider_user_id: String,
        email: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            credential_id,
            provider,
            provider_user_id,
            email,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn credential_id(&self) -> Uuid {
        self.credential_id
    }

    pub fn provider(&self) -> OAuthProvider {
        self.provider
    }

    pub fn provider_user_id(&self) -> &str {
        &self.provider_user_id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
