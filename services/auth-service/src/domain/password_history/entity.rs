use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PasswordHistory {
    id: Uuid,
    credential_id: Uuid,
    password_hash: String,
    created_at: DateTime<Utc>,
}

impl PasswordHistory {
    pub fn new(credential_id: Uuid, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            credential_id,
            password_hash,
            created_at: Utc::now(),
        }
    }

    pub fn from_persisted(
        id: Uuid,
        credential_id: Uuid,
        password_hash: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            credential_id,
            password_hash,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn credential_id(&self) -> Uuid {
        self.credential_id
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
