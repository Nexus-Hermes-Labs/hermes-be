//! Per-request correlation id propagated across HTTP and gRPC.
//!
//! The flow:
//! 1. Inbound HTTP/gRPC requests are wrapped by [`RequestIdScopeLayer`], which
//!    honors the `x-request-id` header (generating a UUIDv4 when absent),
//!    stores the id on the request as a [`HermesRequestId`] extension, and
//!    runs the inner service inside a [`REQUEST_ID`] task-local scope.
//! 2. Application code makes outbound gRPC calls; [`RequestIdInterceptor`]
//!    reads the task-local and injects the `x-request-id` metadata so the
//!    same id flows to the next service.
//!
//! The HTTP `MakeSpan` reads [`HermesRequestId`] from request extensions,
//! so structured logs for every hop carry the same `request_id` field.
//!
//! `RequestIdScopeLayer` carries two [`Service`] impls — one for axum's
//! `http 1.x` `Request` type and one for tonic's `http 0.2` `Request` type —
//! because axum 0.7 and tonic 0.11 link different major versions of the
//! `http` crate.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::task::futures::TaskLocalFuture;
use tokio::task_local;
use tonic::metadata::AsciiMetadataValue;
use tonic::service::Interceptor;
use tonic::Status;
use tower::{Layer, Service};
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

impl<S, B> Service<tonic::codegen::http::Request<B>> for RequestIdScopeService<S>
where
    S: Service<tonic::codegen::http::Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TaskLocalFuture<String, S::Future>;

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
        REQUEST_ID.scope(id, self.inner.call(req))
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

/// Tonic client interceptor that copies the active task-local [`REQUEST_ID`]
/// onto outbound gRPC calls as `x-request-id` metadata. No-op when called
/// outside a request scope (e.g. from background tasks at startup).
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestIdInterceptor;

impl Interceptor for RequestIdInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        if let Some(id) = current_request_id() {
            if let Ok(value) = AsciiMetadataValue::try_from(id.as_str()) {
                request.metadata_mut().insert(REQUEST_ID_HEADER, value);
            }
        }
        Ok(request)
    }
}
