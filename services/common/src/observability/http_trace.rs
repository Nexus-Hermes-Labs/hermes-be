use axum::http::{Request, Response};
use std::time::Duration;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultOnRequest, MakeSpan, OnResponse, TraceLayer};
use tracing::{field, info_span, Span};
use uuid::Uuid;

use crate::observability::request_context::HermesRequestId;

/// `MakeSpan` that opens an `http_request` span carrying the Hermes-standard
/// log fields. Populated at creation: `method`, `uri`, `request_id`. Declared
/// empty (filled later in the request lifecycle): `user_id` (recorded by the
/// `RequestUser` extractor), `status` (recorded on response).
///
/// Reads `request_id` from a `HermesRequestId` extension placed by
/// `RequestIdScopeLayer`. Falls back to generating a UUIDv4 only when that
/// layer is absent (e.g. an integration test that wires up `TraceLayer` alone).
#[derive(Clone, Copy, Debug, Default)]
pub struct HermesMakeSpan;

impl<B> MakeSpan<B> for HermesMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .extensions()
            .get::<HermesRequestId>()
            .map(|id| id.as_str().to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            request_id = %request_id,
            user_id = field::Empty,
            status = field::Empty,
        )
    }
}

/// Records the response status onto the request span and emits a single
/// completion event with latency and status.
#[derive(Clone, Copy, Debug, Default)]
pub struct HermesOnResponse;

impl<B> OnResponse<B> for HermesOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let status = response.status().as_u16();
        span.record("status", status);
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
