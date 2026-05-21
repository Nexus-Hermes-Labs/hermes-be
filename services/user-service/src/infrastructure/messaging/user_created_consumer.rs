use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use tracing::info;
use uuid::Uuid;

use common::domain::event::EventEnvelope;
use common::infrastructure::outbox::JetStreamEventHandler;

#[derive(Debug, Deserialize)]
pub struct UserCreatedPayload {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
}

pub struct UserCreatedHandler;

#[async_trait]
impl JetStreamEventHandler for UserCreatedHandler {
    type Event = UserCreatedPayload;

    fn subject(&self) -> &str {
        "user.created"
    }

    fn durable_name(&self) -> &str {
        "user-service-user-created"
    }

    async fn handle(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        envelope: EventEnvelope<Self::Event>,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            INSERT INTO user_profiles (
                id, username, display_name, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, 'offline', NOW(), NOW())
            "#,
        )
        .bind(envelope.payload.user_id)
        .bind(&envelope.payload.username)
        .bind(&envelope.payload.display_name)
        .execute(&mut **tx)
        .await?;

        info!(
            user_id = %envelope.payload.user_id,
            username = %envelope.payload.username,
            email = %envelope.payload.email,
            aggregate_id = %envelope.aggregate_id,
            occurred_at = %envelope.occurred_at,
            "User profile created from event"
        );

        Ok(())
    }
}
