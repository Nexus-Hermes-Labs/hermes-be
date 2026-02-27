//! Fluent builder that wires all dependencies into an [`AppState`] and a
//! [`Server`].

use std::sync::Arc;

use common::infrastructure::security::jwt_manager::JwtManager;
use common::observability::Metrics;
use dashmap::DashMap;
use tracing::info;

use crate::bootstrap::error::BootstrapError;
use crate::presentation::http::server::Server;
use crate::state::{AppState, ClientRegistry, MsgContextCache, SubscriptionRegistry};

/// Builder for the realtime-service application.
#[allow(missing_debug_implementations)]
pub struct AppBuilder {
    service_name: Option<&'static str>,
    nats: Option<async_nats::Client>,
    jwt_manager: Option<JwtManager>,
    metrics: Option<Metrics>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    /// Create an unconfigured builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            service_name: None,
            nats: None,
            jwt_manager: None,
            metrics: None,
        }
    }

    /// Set the service name (used in tracing / metrics labels).
    #[must_use]
    pub const fn with_service_name(mut self, name: &'static str) -> Self {
        self.service_name = Some(name);
        self
    }

    /// Provide the NATS async client.
    #[must_use]
    pub fn with_nats(mut self, nats: async_nats::Client) -> Self {
        self.nats = Some(nats);
        self
    }

    /// Provide the JWT manager used to authenticate WebSocket connections.
    #[must_use]
    pub fn with_jwt_manager(mut self, mgr: JwtManager) -> Self {
        self.jwt_manager = Some(mgr);
        self
    }

    /// Provide the Prometheus metrics collector.
    #[must_use]
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Build the [`Server`] and the fully-wired [`AppState`].
    ///
    /// The caller is responsible for spawning the NATS listener task using the
    /// returned `AppState`.
    pub fn build(self) -> Result<(Server, AppState), BootstrapError> {
        let _service_name = self.service_name.ok_or_else(|| {
            BootstrapError::Initialization("service name must be set".to_string())
        })?;
        let nats = self.nats.ok_or_else(|| {
            BootstrapError::Initialization("NATS client must be set".to_string())
        })?;
        let jwt_manager = self.jwt_manager.ok_or_else(|| {
            BootstrapError::Initialization("JWT manager must be set".to_string())
        })?;
        let metrics = self.metrics.ok_or_else(|| {
            BootstrapError::Initialization("metrics must be set".to_string())
        })?;

        // ── Shared registries ──────────────────────────────────────────────
        let client_registry: Arc<ClientRegistry> = Arc::new(DashMap::new());
        let sub_registry: Arc<SubscriptionRegistry> = Arc::new(DashMap::new());
        let msg_ctx_cache: Arc<MsgContextCache> = Arc::new(DashMap::new());

        info!("✅ Registries initialised");

        // ── Application state ──────────────────────────────────────────────
        let app_state = AppState {
            client_registry,
            sub_registry,
            msg_ctx_cache,
            nats,
            jwt_manager: Arc::new(jwt_manager),
            metrics,
        };

        // ── HTTP server ────────────────────────────────────────────────────
        let server = Server::new(app_state.clone());
        info!("✅ HTTP server ready");

        Ok((server, app_state))
    }
}
