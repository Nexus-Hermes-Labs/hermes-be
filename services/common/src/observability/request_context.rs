//! Per-request correlation id and W3C trace context propagated across HTTP
//! and gRPC.
//!
//! The flow:
//! 1. Inbound HTTP/gRPC requests are wrapped by [`RequestIdScopeLayer`], which
//!    honors the `x-request-id` header (generating a UUIDv4 when absent),
//!    stores the id on the request as a [`HermesRequestId`] extension, and
//!    runs the inner service inside a [`REQUEST_ID`] task-local scope.
//! 2. Application code makes outbound gRPC calls; [`RequestIdInterceptor`]
//!    reads the task-local and injects both `x-request-id` and the W3C
//!    `traceparent` (from the current OTel context) so the same correlation
//!    id *and* trace context flow to the next service.
//!
//! The HTTP `MakeSpan` reads [`HermesRequestId`] from request extensions,
//! and extracts the inbound `traceparent` to set as the parent of the local
//! span — closing the loop on cross-service trace continuity.
//!
//! `RequestIdScopeLayer` carries two [`Service`] impls — one for axum's
//! `http 1.x` `Request` type and one for tonic's `http 0.2` `Request` type —
//! because axum 0.7 and tonic 0.11 link different major versions of the
//! `http` crate.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use opentelemetry::{
    propagation::{Extractor, Injector},
    trace::{TraceContextExt, TraceId},
};
use tokio::task::futures::TaskLocalFuture;
use tokio::task_local;
use tonic::metadata::{AsciiMetadataValue, KeyRef, MetadataKey, MetadataMap};
use tonic::service::Interceptor;
use tonic::Status;
use tower::{Layer, Service};
use tracing::{field, info_span, Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

/// HTTP/gRPC header used to carry the request correlation id between services.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

task_local! {
    /// Active request id for the current task. Set by [`RequestIdScopeLayer`]
    /// on every inbound request; read by [`RequestIdInterceptor`] when
    /// initiating an outbound gRPC call.
    pub static REQUEST_ID: String;
}

/// Returns the current request id if running inside a request scope.
#[must_use]
pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(Clone::clone).ok()
}

/// Request id stored on the request via `extensions_mut().insert(...)` so
/// downstream code (notably `HermesMakeSpan`) can read it without re-parsing
/// headers.
#[derive(Clone, Debug)]
pub struct HermesRequestId(pub String);

impl HermesRequestId {
    /// Borrow the id as a `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tower [`Layer`] that ensures every inbound request carries an
/// `x-request-id` and runs the inner service inside a [`REQUEST_ID`]
/// task-local scope.
///
/// Reads the inbound `x-request-id` header, generates a UUIDv4 when absent,
/// stores the value as a [`HermesRequestId`] extension on the request, then
/// drives the inner service inside [`REQUEST_ID::scope`]. Carries separate
/// [`Service`] impls for axum and tonic so the same layer composes onto both.
///
/// Echoing the request id back on the HTTP response is handled separately by
/// [`PropagateRequestIdResponseLayer`] — it is HTTP-only and intentionally
/// not part of this layer so the gRPC server can use a single, minimal layer.
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestIdScopeLayer;

impl<S> Layer<S> for RequestIdScopeLayer {
    type Service = RequestIdScopeService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdScopeService { inner }
    }
}

/// Service produced by [`RequestIdScopeLayer`]. See module docs.
#[derive(Clone, Debug)]
pub struct RequestIdScopeService<S> {
    inner: S,
}

/// Pull `x-request-id` out of `headers` (returning a `String`), or generate a
/// fresh UUIDv4 when the header is missing or non-ASCII.
fn extract_or_generate_request_id<'a, I>(headers: I) -> String
where
    I: IntoIterator<Item = (&'a [u8], &'a [u8])>,
{
    for (name, value) in headers {
        if name.eq_ignore_ascii_case(REQUEST_ID_HEADER.as_bytes()) {
            if let Ok(s) = std::str::from_utf8(value) {
                return s.to_owned();
            }
        }
    }
    Uuid::new_v4().to_string()
}

// ---------------------------------------------------------------------------
// axum (http 1.x) impl
// ---------------------------------------------------------------------------

impl<S, B> Service<axum::http::Request<B>> for RequestIdScopeService<S>
where
    S: Service<axum::http::Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TaskLocalFuture<String, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: axum::http::Request<B>) -> Self::Future {
        let id = extract_or_generate_request_id(
            req.headers()
                .iter()
                .map(|(n, v)| (n.as_str().as_bytes(), v.as_bytes())),
        );
        req.extensions_mut().insert(HermesRequestId(id.clone()));
        REQUEST_ID.scope(id, self.inner.call(req))
    }
}

// ---------------------------------------------------------------------------
// tonic (http 0.2) impl
// ---------------------------------------------------------------------------
//
// Tonic's `Server::layer()` runs *before* the gRPC method dispatcher, so this
// is where we extract the inbound `traceparent` and open a `grpc_request`
// span linked to the parent context. The handler future runs inside that
// span, so any tracing inside the handler is auto-parented to the same
// distributed trace.

impl<S, B> Service<tonic::codegen::http::Request<B>> for RequestIdScopeService<S>
where
    S: Service<tonic::codegen::http::Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TaskLocalFuture<String, tracing::instrument::Instrumented<S::Future>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: tonic::codegen::http::Request<B>) -> Self::Future {
        let id = extract_or_generate_request_id(
            req.headers()
                .iter()
                .map(|(n, v)| (n.as_str().as_bytes(), v.as_bytes())),
        );
        req.extensions_mut().insert(HermesRequestId(id.clone()));

        let span = info_span!(
            "grpc_request",
            uri = %req.uri(),
            request_id = %id,
            trace_id = field::Empty,
        );

        // Extract inbound W3C trace context from the gRPC headers and link
        // this span to it. Falls through cleanly to a noop when no
        // propagator is installed (log-only mode).
        let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&Http02HeaderExtractor(req.headers()))
        });
        span.set_parent(parent_cx);

        // Surface the trace_id on the tracing span so JSON log lines emitted
        // by the handler carry it for Loki↔Tempo correlation.
        let cx = span.context();
        let otel_span = cx.span();
        let span_ctx = otel_span.span_context();
        if span_ctx.is_valid() && span_ctx.trace_id() != TraceId::INVALID {
            span.record("trace_id", span_ctx.trace_id().to_string());
        }

        REQUEST_ID.scope(id, self.inner.call(req).instrument(span))
    }
}

// ---------------------------------------------------------------------------
// HTTP-only response-header propagation
// ---------------------------------------------------------------------------

/// HTTP-only [`Layer`] that copies [`HermesRequestId`] from the request
/// extensions onto the outgoing response as an `x-request-id` header.
///
/// Apply *inside* [`RequestIdScopeLayer`] so the extension is already set
/// when this runs.
#[derive(Clone, Copy, Debug, Default)]
pub struct PropagateRequestIdResponseLayer;

impl<S> Layer<S> for PropagateRequestIdResponseLayer {
    type Service = PropagateRequestIdResponseService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PropagateRequestIdResponseService { inner }
    }
}

/// Service produced by [`PropagateRequestIdResponseLayer`].
#[derive(Clone, Debug)]
pub struct PropagateRequestIdResponseService<S> {
    inner: S,
}

impl<S, ReqB, ResB> Service<axum::http::Request<ReqB>> for PropagateRequestIdResponseService<S>
where
    S: Service<axum::http::Request<ReqB>, Response = axum::http::Response<ResB>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqB>) -> Self::Future {
        let response_id = req
            .extensions()
            .get::<HermesRequestId>()
            .and_then(|id| axum::http::HeaderValue::from_str(id.as_str()).ok());

        let fut = self.inner.call(req);
        Box::pin(async move {
            let mut response = fut.await?;
            if let Some(value) = response_id {
                response.headers_mut().insert(REQUEST_ID_HEADER, value);
            }
            Ok(response)
        })
    }
}

// ---------------------------------------------------------------------------
// gRPC client interceptor
// ---------------------------------------------------------------------------

/// Tonic client interceptor that propagates per-request correlation context
/// onto outbound gRPC calls. Two pieces of state get injected:
///
/// - `x-request-id` from the task-local [`REQUEST_ID`] scope, so logs from
///   downstream services link to the same id used in this hop.
/// - W3C `traceparent` (and `tracestate` when present) from the current
///   OpenTelemetry context, so distributed-trace continuity survives the
///   gRPC boundary.
///
/// Both injections are no-ops when called outside a request scope (e.g.
/// from background tasks at startup) or when no OTel propagator is installed
/// (log-only mode without an OTLP endpoint).
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestIdInterceptor;

impl Interceptor for RequestIdInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        if let Some(id) = current_request_id() {
            if let Ok(value) = AsciiMetadataValue::try_from(id.as_str()) {
                request.metadata_mut().insert(REQUEST_ID_HEADER, value);
            }
        }

        // Inject the current OTel span context as `traceparent` metadata.
        // Pulled from `tracing::Span::current()` so the source of truth is
        // the active tracing span (which is what every other observability
        // hook reads from — keeps drift impossible).
        let cx = Span::current().context();
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut TonicMetadataInjector(request.metadata_mut()));
        });

        Ok(request)
    }
}

// ---------------------------------------------------------------------------
// OpenTelemetry header adapters for tonic metadata
// ---------------------------------------------------------------------------

/// `Injector` so the global `TextMapPropagator` can write `traceparent` /
/// `tracestate` into a tonic [`MetadataMap`]. Binary-only metadata keys
/// (the `-bin` suffix convention) are ignored — the W3C propagator only
/// produces ASCII headers.
pub(crate) struct TonicMetadataInjector<'a>(pub(crate) &'a mut MetadataMap);

impl<'a> Injector for TonicMetadataInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = AsciiMetadataValue::try_from(value) {
                self.0.insert(name, val);
            }
        }
    }
}

/// `Extractor` for inbound tonic metadata — kept for parity with the
/// injector even if no current call site uses it directly. The gRPC server
/// path actually extracts via [`Http02HeaderExtractor`] because `Server::layer`
/// hands us the raw `tonic::codegen::http::Request`, not a `tonic::Request`.
#[allow(dead_code)]
pub(crate) struct TonicMetadataExtractor<'a>(pub(crate) &'a MetadataMap);

impl<'a> Extractor for TonicMetadataExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .filter_map(|k| match k {
                KeyRef::Ascii(s) => Some(s.as_str()),
                KeyRef::Binary(_) => None,
            })
            .collect()
    }
}

/// Adapter so the global `TextMapPropagator` can read trace context from
/// the `http 0.2` `HeaderMap` that tonic 0.11 hands the server-side tower
/// layer. Mirrors `AxumHeaderExtractor` in `http_trace.rs` — we keep two
/// because axum 0.7 and tonic 0.11 link different `http` major versions.
pub(crate) struct Http02HeaderExtractor<'a>(pub(crate) &'a tonic::codegen::http::HeaderMap);

impl<'a> Extractor for Http02HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}
