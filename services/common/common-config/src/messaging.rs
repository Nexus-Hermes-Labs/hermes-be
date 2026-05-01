use serde::Deserialize;

/// NATS configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MessagingConfig {
    pub servers: String, // Separation with comma: "127.0.0.1:4222,127.0.0.1:4223"
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub max_reconnect_attempts: usize,
}

impl Default for MessagingConfig {
    fn default() -> Self {
        Self {
            servers: "127.0.0.1:4222".to_string(),
            username: None,
            password: None,
            token: None,
            max_reconnect_attempts: 10,
        }
    }
}

impl MessagingConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            servers: std::env::var("NATS_SERVERS").unwrap_or_else(|_| "127.0.0.1:4222".to_string()),
            username: std::env::var("NATS_USER").ok(),
            password: std::env::var("NATS_PASSWORD").ok(),
            token: std::env::var("NATS_TOKEN").ok(),
            max_reconnect_attempts: std::env::var("NATS_MAX_RECONNECT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        })
    }

    pub fn get_url(&self) -> String {
        self.servers.clone()
    }
}
