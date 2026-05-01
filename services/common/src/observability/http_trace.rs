use axum::http::{HeaderMap, Request, Response};
use metrics::{counter, histogram};
use opentelemetry::{
    propagation::Extractor,
    trace::{TraceContextExt, TraceId},
};
use std::time::Duration;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultOnRequest, MakeSpan, OnResponse, TraceLayer};
use tracing::{field, info_span, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

use crate::observability::request_context::HermesRequestId;

/// `MakeSpan` that opens an `http_request` span carrying the Hermes-standard
/// log fields. Populated at creation: `method`, `uri`, `request_id`, and
/// (when an OTLP endpoint is configured and the inbound request carries a
/// W3C `traceparent`) `trace_id`. Declared empty (filled later in the request
/// lifecycle): `user_id` (recorded by the `RequestUser` extractor), `status`
/// (recorded on response).
///
/// Reads `request_id` from a `HermesRequestId` extension placed by
/// `RequestIdScopeLayer`. Falls back to generating a UUIDv4 only when that
/// layer is absent (e.g. an integration test that wires up `TraceLayer` alone).
///
/// The span is also linked to the inbound OpenTelemetry trace context (via
/// W3C `traceparent`/`tracestate` headers) so distributed-trace continuity is
/// preserved across the network boundary. When tracing is in log-only mode
/// (no OTLP endpoint set) this is a no-op — the global propagator is a noop
/// and the extracted context is empty, leaving `trace_id` blank.
#[derive(Clone, Copy, Debug, Default)]
pub struct HermesMakeSpan;

impl<B> MakeSpan<B> for HermesMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .extensions()
            .get::<HermesRequestId>()
            .map(|id| id.as_str().to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let span = info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            request_id = %request_id,
            trace_id = field::Empty,
            user_id = field::Empty,
            status = field::Empty,
        );

        // Link this span to whatever OTel context the caller carried in via
        // W3C `traceparent`. When no global propagator is installed (log-only
        // mode), this returns an empty context and `set_parent` is a no-op.
        let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&AxumHeaderExtractor(request.headers()))
        });
        span.set_parent(parent_cx);

        // Record `trace_id` on the tracing span as a hex string so JSON log
        // lines carry it. Grafana's Loki datasource has a derived field that
        // matches this and turns it into a clickable Tempo link.
        let cx = span.context();
        let otel_span = cx.span();
        let span_ctx = otel_span.span_context();
        if span_ctx.is_valid() && span_ctx.trace_id() != TraceId::INVALID {
            span.record("trace_id", span_ctx.trace_id().to_string());
        }

        span
    }
}

/// Adapter so `opentelemetry::propagation::TextMapPropagator::extract` can
/// read from axum's `http 1.x` `HeaderMap`. `opentelemetry-http` exists for
/// this but its current release links a different `http` major than axum
/// 0.7 — easier to write the four-line adapter ourselves.
struct AxumHeaderExtractor<'a>(&'a HeaderMap);

impl<'a> Extractor for AxumHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// Records the response status onto the request span, emits a single
/// completion event with latency and status, and increments the
/// `http_request_duration_seconds` histogram + `http_requests_total`
/// counter so Prometheus can drive per-service latency / error-rate panels.
///
/// Labels are kept to `status` only — per-service breakdown comes from
/// Prometheus's automatic `job` / `instance` labels, and per-method labels
/// are skipped so cardinality stays bounded as routes proliferate.
#[derive(Clone, Copy, Debug, Default)]
pub struct HermesOnResponse;

impl<B> OnResponse<B> for HermesOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let status = response.status().as_u16();
        span.record("status", status);

        let status_label = status.to_string();
        histogram!(
            "http_request_duration_seconds",
            "status" => status_label.clone()
        )
        .record(latency.as_secs_f64());
        counter!(
            "http_requests_total",
            "status" => status_label
        )
        .increment(1);

        tracing::info!(
            latency_ms = latency.as_millis() as u64,
            status,
            "http_request completed",
        );
    }
}

/// The exact `TraceLayer` type used by every Hermes service. Exposed so service
/// `routes` modules can declare it as a parameter type.
pub type HermesTraceLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    HermesMakeSpan,
    DefaultOnRequest,
    HermesOnResponse,
>;

/// Pre-configured HTTP `TraceLayer`. Apply to every service router with `.layer(...)`.
#[must_use]
pub fn request_trace_layer() -> HermesTraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(HermesMakeSpan)
        .on_response(HermesOnResponse)
}
