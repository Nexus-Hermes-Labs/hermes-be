use serde::Deserialize;

/// Cache configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: i32,
    pub max_connections: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            username: None,
            password: None,
            database: 0,
            max_connections: 10,
        }
    }
}

impl CacheConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            host: std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("REDIS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(6379),
            username: std::env::var("REDIS_USER").ok(),
            password: std::env::var("REDIS_PASSWORD").ok(),
            database: std::env::var("REDIS_DB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            max_connections: std::env::var("REDIS_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        })
    }

    /// Build redis connection string
    /// Format: redis://[user:password@]host:port/db
    pub fn get_url(&self) -> String {
        let auth = match (&self.username, &self.password) {
            (Some(user), Some(pass)) => format!("{}:{}@", user, pass),
            (None, Some(pass)) => format!(":{}@", pass),
            _ => "".to_string(),
        };

        format!(
            "redis://{}{}:{}/{}",
            auth, self.host, self.port, self.database
        )
    }
}
