// common-config/src/lib.rs
pub mod cache;
pub mod database;
pub mod error;
pub mod grpc_endpoints;
pub mod logging;
pub mod messaging;
pub mod secrets;
pub mod service;
pub mod smtp;

pub use cache::CacheConfig;
pub use database::DatabaseConfig;
pub use grpc_endpoints::GrpcEndpointsConfig;
pub use logging::LoggingConfig;
pub use messaging::MessagingConfig;
pub use secrets::SecretsConfig;
pub use service::ServiceConfig;
pub use smtp::SmtpConfig;

use crate::error::ConfigError;
use figment::{
    providers::{Env, Format, Json},
    Figment,
};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;
use std::time::Duration;

/// Default Consul URL when running inside the docker-compose network.
/// Override per-environment via the `CONSUL_URL` env var.
const DEFAULT_CONSUL_URL: &str = "http://hermes-consul:8500";

/// Hard timeout for individual Consul KV reads. Bootstrap blocks on these,
/// so we cap each call so a wedged Consul cannot stall service startup
/// indefinitely.
const CONSUL_TIMEOUT: Duration = Duration::from_secs(5);

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub service: ServiceConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub redis: CacheConfig,
    #[serde(default)]
    pub nats: MessagingConfig,
    #[serde(default)]
    pub smtp: SmtpConfig,
    #[serde(default)]
    pub secrets: SecretsConfig,
    #[serde(default)]
    pub grpc_endpoints: GrpcEndpointsConfig,
}

impl Config {
    /// Load configuration with the merge order (last layer wins):
    ///
    /// 1. Consul KV `config/application/data` — shared cross-service baseline.
    /// 2. Consul KV `config/{service_name}/data` — service-specific baseline.
    /// 3. `.env` files (workspace root, then per-service) — developer local overrides.
    /// 4. Environment variables (`APP_*`) — what compose/CI/host actually sets at boot.
    ///
    /// **Why env wins over Consul.** Hermes runs in two modes:
    /// - `make up` (containers): compose sets `APP_DATABASE__HOST=hermes-postgres` etc.
    /// - `make run-{service}` (host): developer's `.env` sets localhost equivalents.
    ///
    /// Consul is the same checked-in baseline for both modes, so the
    /// mode-specific layer (env vars / dotenvy) has to win. This makes Consul
    /// the *fallback*: when neither env nor `.env` defines a key, Consul fills
    /// it. Operators changing values in Consul still need to restart the
    /// affected service — `Config::load` only runs once, at startup.
    ///
    /// When Consul is unreachable, [`fetch_consul_kv`] logs to stderr and
    /// returns `None`; the env layer alone still yields a complete config.
    pub fn load(service_name: &str) -> Result<Self, ConfigError> {
        // 1. Force set the service name in the environment so .env files don't
        // need to hardcode it.
        env::set_var("APP_SERVICE__NAME", service_name);

        // 2. Build figment baseline-to-override:
        //    Consul (application) → Consul (service) → dotenvy → env
        let mut figment = Figment::new();

        let consul_url =
            env::var("CONSUL_URL").unwrap_or_else(|_| DEFAULT_CONSUL_URL.to_string());

        if let Some(json) = fetch_consul_kv(&consul_url, "application") {
            figment = figment.merge(Json::string(&json));
        }
        if let Some(json) = fetch_consul_kv(&consul_url, service_name) {
            figment = figment.merge(Json::string(&json));
        }

        // 3. dotenvy populates the process env from `.env` files. Has to run
        // *before* the Env provider reads so its values are visible.
        dotenvy::dotenv().ok();
        let service_env_path = format!("services/{}/.env", service_name);
        dotenvy::from_filename(&service_env_path).ok();

        // 4. Env vars are the authoritative final layer — compose, CI, or
        // dotenvy-populated process env all funnel through here.
        figment = figment.merge(Env::prefixed("APP_").split("__"));

        let config: Config = figment
            .extract()
            .map_err(|e| ConfigError::Extraction(e.to_string()))?;

        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate service config
        if self.service.name.is_empty() {
            return Err(ConfigError::Validation(
                "Service name cannot be empty".into(),
            ));
        }

        if self.service.port == 0 {
            return Err(ConfigError::Validation(
                "Service port cannot be 0. Please define APP_SERVICE__PORT.".into(),
            ));
        }

        if let Some(grpc_port) = self.service.grpc_port {
            if grpc_port == 0 {
                return Err(ConfigError::Validation(
                    "gRPC port cannot be 0. Please define APP_SERVICE__GRPC_PORT.".into(),
                ));
            }

            if grpc_port == self.service.port {
                return Err(ConfigError::Validation(
                    "gRPC port must be different from HTTP port.".into(),
                ));
            }
        }

        // Validate database config
        if self.database.host.is_empty() {
            return Err(ConfigError::Validation(
                "Database host cannot be empty".into(),
            ));
        }

        // Validate Redis config
        if self.redis.host.is_empty() {
            return Err(ConfigError::Validation("Redis host cannot be empty".into()));
        }

        // Validate NATS config
        if self.nats.servers.is_empty() {
            return Err(ConfigError::Validation(
                "NATS servers cannot be empty".into(),
            ));
        }

        // Validate SMTP config
        if self.smtp.host.is_empty() {
            return Err(ConfigError::Validation("SMTP host cannot be empty".into()));
        }

        if self.smtp.from_address.is_empty() {
            return Err(ConfigError::Validation(
                "SMTP from_address cannot be empty".into(),
            ));
        }

        // Delegate nested secrets validations
        self.secrets
            .jwt
            .validate()
            .map_err(|e| ConfigError::Validation(format!("JWT config error: {}", e)))?;

        self.secrets
            .password
            .validate()
            .map_err(|e| ConfigError::Validation(format!("Password config error: {}", e)))?;

        Ok(())
    }

    /// Helper to determine if this is a gateway service
    pub fn is_gateway(&self) -> bool {
        self.service.name == "gateway-service"
    }

    /// Get service URL for internal communication
    pub fn service_url(&self) -> String {
        format!("http://{}:{}", self.service.host, self.service.port)
    }
}

/// Fetch a single Consul KV entry as a raw JSON string.
///
/// Returns `None` when Consul is unreachable, when the key is missing, or
/// when the response body is empty. All failure modes log a warning to
/// stderr — `tracing` isn't initialised yet during config bootstrap, so we
/// can't use the structured logger.
///
/// Uses `ureq` (synchronous, no internal runtime) rather than
/// `reqwest::blocking` because `Config::load` is called from inside the
/// tokio runtime started by `#[tokio::main]`. `reqwest::blocking` warns
/// about deadlocks in that context; `ureq` does its own threadless blocking
/// I/O and is safe.
fn fetch_consul_kv(consul_url: &str, key: &str) -> Option<String> {
    let url = format!("{}/v1/kv/config/{}/data?raw", consul_url, key);

    let agent = ureq::AgentBuilder::new()
        .timeout(CONSUL_TIMEOUT)
        .build();

    match agent.get(&url).call() {
        Ok(response) if response.status() == 200 => match response.into_string() {
            Ok(body) if !body.is_empty() => Some(body),
            Ok(_) => {
                eprintln!(
                    "[config] Consul key '{}' returned empty body — falling back to env baseline",
                    key
                );
                None
            }
            Err(err) => {
                eprintln!(
                    "[config] Consul key '{}' body read failed ({}): falling back to env baseline",
                    key, err
                );
                None
            }
        },
        // 404 is the normal "not yet migrated" case — quiet it.
        Ok(response) if response.status() == 404 => None,
        Ok(response) => {
            eprintln!(
                "[config] Consul key '{}' returned status {}: falling back to env baseline",
                key,
                response.status()
            );
            None
        }
        Err(ureq::Error::Status(_, _)) => None,
        Err(ureq::Error::Transport(err)) => {
            eprintln!(
                "[config] Consul unreachable at {} ({}): falling back to env baseline",
                consul_url, err
            );
            None
        }
    }
}

// Global static config instance
static CONFIG: OnceCell<Config> = OnceCell::new();

/// Initialize global configuration
pub fn init_config(service_name: &str) -> Result<(), ConfigError> {
    let config = Config::load(service_name)?;
    CONFIG
        .set(config)
        .map_err(|_| ConfigError::Validation("Config already initialized".into()))?;
    Ok(())
}

/// Get reference to global configuration
pub fn config() -> &'static Config {
    CONFIG
        .get()
        .expect("CONFIG is not initialized. Call init_config() first")
}

/// Check if config is initialized
pub fn is_initialized() -> bool {
    CONFIG.get().is_some()
}
