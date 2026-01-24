use async_nats::Client;
use crate::{AppError, Event, Result};

pub struct MessageQueue {
    client: Client,
}

impl MessageQueue {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn publish(&self, event: &Event) -> Result<()> {
        let topic = event.topic();
        let payload = serde_json::to_vec(event)
            .map_err(|e| AppError::MessageQueue(e.to_string()))?;
        
        self.client.publish(topic, payload.into())
            .await
            .map_err(|e| AppError::MessageQueue(e.to_string()))?;
        
        Ok(())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber> {
        self.client.subscribe(subject.to_string())
            .await
            .map_err(|e| AppError::MessageQueue(e.to_string()))
    }
}
