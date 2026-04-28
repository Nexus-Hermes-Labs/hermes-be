use axum::routing::get;
use axum::Router;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;

/// Global handle to the Prometheus recorder. Populated once by [`Metrics::init`].
/// Read by [`metrics_routes`] so each service can expose `/metrics` without
/// threading state through its router constructor.
static METRICS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

/// Owns the Prometheus recorder for a service. Construct exactly once in
/// `bootstrap` so the global recorder + render handle are installed before
/// any metric emit site runs.
#[derive(Clone)]
pub struct Metrics {
    handle: PrometheusHandle,
}

impl std::fmt::Debug for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metrics").finish_non_exhaustive()
    }
}

impl Metrics {
    /// Install the Prometheus recorder globally and stash the render handle.
    /// Calling this more than once is a programmer error and returns the
    /// underlying `install_recorder` failure.
    pub fn init() -> Result<Self, anyhow::Error> {
        let handle = PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("http_request_duration_seconds".to_string()),
                &[
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ],
            )?
            .install_recorder()?;

        // Best-effort: idempotent across hot-reload-style scenarios. If a
        // handle is already present (e.g. integration tests) we keep the
        // first one — both render the same global recorder.
        let _ = METRICS_HANDLE.set(handle.clone());

        Ok(Self { handle })
    }

    /// Render the current Prometheus exposition payload as a string.
    pub fn render(&self) -> String {
        self.handle.render()
    }
}

/// Stateless `/metrics` router. Service routers nest this at `/metrics`
/// after [`Metrics::init`] has run during bootstrap.
#[must_use]
pub fn metrics_routes() -> Router {
    Router::new().route("/", get(render_metrics))
}

async fn render_metrics() -> String {
    METRICS_HANDLE
        .get()
        .map(PrometheusHandle::render)
        .unwrap_or_default()
}
