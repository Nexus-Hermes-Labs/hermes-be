use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

pub struct Metrics {
    handle: PrometheusHandle,
}

impl Metrics {
    pub fn init() -> Result<Self, anyhow::Error> {
        let handle = PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("http_request_duration_seconds".to_string()),
                &[
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ],
            )?
            .install_recorder()?;

        Ok(Self { handle })
    }

    pub fn render(&self) -> String {
        self.handle.render()
    }
}
