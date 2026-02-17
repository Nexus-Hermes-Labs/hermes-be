use super::error::MessagingError;
use super::event_publisher::EventPublisher;
use crate::domain::event::EventEnvelope;
use async_nats::Client;
use async_trait::async_trait;
use serde::Serialize;
use tracing::{debug, error, info};

/// NATS event publisher
#[derive(Clone)]
pub struct NatsEventPublisher {
    client: Client,
    service_name: String,
}

impl NatsEventPublisher {
    /// Create new NATS publisher
    pub async fn new(service_name: &'static str, nats_url: &str) -> Result<Self, MessagingError> {
        info!(nats_url = %nats_url, "Connecting to NATS");

        let client = async_nats::connect(nats_url).await.map_err(|e| {
            error!(error = %e, "Failed to connect to NATS");
            MessagingError::ConnectionFailed(e.to_string())
        })?;

        info!("Successfully connected to NATS");

        Ok(Self {
            client,
            service_name: service_name.into(),
        })
    }

    /// Create with existing client
    pub fn with_client(service_name: impl Into<String>, client: Client) -> Self {
        Self {
            client,
            service_name: service_name.into(),
        }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

#[async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn publish_bytes(
        &self,
        subject: &str,
        payload: Vec<u8>,
    ) -> Result<(), MessagingError> {
        debug!(
            subject = %subject,
            "Publishing raw bytes to NATS"
        );

        self.client
            .publish(subject.to_string(), payload.into())
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    subject = %subject,
                    "Failed to publish event"
                );
                MessagingError::PublishFailed(e.to_string())
            })?;

        Ok(())
    }

    async fn health_check(&self) -> Result<(), MessagingError> {
        let subject = format!("_healthcheck.{}", self.service_name);

        self.client
            .publish(subject, "ping".into())
            .await
            .map_err(|e| MessagingError::ConnectionFailed(e.to_string()))?;

        debug!("NATS health check passed");
        Ok(())
    }
}
