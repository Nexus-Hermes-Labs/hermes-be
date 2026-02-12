use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub auth_service_url: String,
    pub user_service_url: String,
    pub channel_service_url: String,
    pub chat_service_url: String,
    pub voice_service_url: String,
    pub stream_service_url: String,
    pub presence_service_url: String,

    #[serde(default = "default_timeout_secs")]
    pub service_timeout_secs: u64,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,

    #[serde(default = "default_circuit_breaker_threshold")]
    pub circuit_breaker_threshold: u32,

    #[serde(default = "default_circuit_breaker_timeout")]
    pub circuit_breaker_timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay_ms() -> u64 {
    100
}

fn default_circuit_breaker_threshold() -> u32 {
    5
}

fn default_circuit_breaker_timeout() -> u64 {
    60
}

impl GatewayConfig {
    /// Get service URL by service name
    pub fn get_service_url(&self, service: &str) -> Option<&str> {
        match service {
            "auth" => Some(&self.auth_service_url),
            "user_profile" => Some(&self.user_service_url),
            "channel" => Some(&self.channel_service_url),
            "chat" => Some(&self.chat_service_url),
            "voice" => Some(&self.voice_service_url),
            "stream" => Some(&self.stream_service_url),
            "presence" => Some(&self.presence_service_url),
            _ => None,
        }
    }

    /// Get all service URLs as a HashMap
    pub fn all_service_urls(&self) -> HashMap<String, String> {
        let mut services = HashMap::new();
        services.insert("auth".to_string(), self.auth_service_url.clone());
        services.insert("user_profile".to_string(), self.user_service_url.clone());
        services.insert("channel".to_string(), self.channel_service_url.clone());
        services.insert("chat".to_string(), self.chat_service_url.clone());
        services.insert("voice".to_string(), self.voice_service_url.clone());
        services.insert("stream".to_string(), self.stream_service_url.clone());
        services.insert("presence".to_string(), self.presence_service_url.clone());
        services
    }

    /// Get service timeout as Duration
    pub fn service_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.service_timeout_secs)
    }

    /// Get retry delay as Duration
    pub fn retry_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.retry_delay_ms)
    }

    /// Get circuit breaker timeout as Duration
    pub fn circuit_breaker_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.circuit_breaker_timeout_secs)
    }
}
