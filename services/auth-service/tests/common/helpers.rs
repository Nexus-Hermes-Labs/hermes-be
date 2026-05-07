use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Send a JSON request to the router and return status + parsed body.
pub async fn make_json_request(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    make_json_request_with_headers(app, method, uri, body, &[]).await
}

/// Same as `make_json_request` but allows extra request headers.
/// Used to inject `X-Forwarded-For` for client-IP capture tests.
pub async fn make_json_request_with_headers(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<serde_json::Value>,
    extra_headers: &[(&str, &str)],
) -> (StatusCode, serde_json::Value) {
    let body_str = body
        .map(|v| serde_json::to_string(&v).expect("serialize body"))
        .unwrap_or_default();

    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    for (k, v) in extra_headers {
        builder = builder.header(*k, *v);
    }
    let request = builder
        .body(axum::body::Body::from(body_str))
        .expect("build request");

    let response = app.oneshot(request).await.expect("send request");
    let status = response.status();

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();

    let value: serde_json::Value = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            // Response wasn't JSON (e.g. plain-text axum rejection) — wrap it
            serde_json::Value::String(String::from_utf8_lossy(&bytes).into_owned())
        })
    };

    (status, value)
}
