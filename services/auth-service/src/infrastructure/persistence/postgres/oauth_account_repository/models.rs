use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::oauth_account::{OAuthAccount, OAuthProvider};

/// Database row for the `oauth_accounts` table.
#[derive(Debug, Clone, FromRow)]
pub struct OAuthAccountRow {
    pub id: Uuid,
    pub credential_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<OAuthAccountRow> for OAuthAccount {
    type Error = String;

    fn try_from(row: OAuthAccountRow) -> Result<Self, Self::Error> {
        let provider = row
            .provider
            .parse::<OAuthProvider>()
            .map_err(|e| format!("Invalid oauth provider in database: {:?}", e))?;

        Ok(OAuthAccount::from_persisted(
            row.id,
            row.credential_id,
            provider,
            row.provider_user_id,
            row.email,
            row.created_at,
        ))
    }
}
