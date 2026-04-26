use crate::infrastructure::security::jwt_manager::SystemRole;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::str::FromStr;
use uuid::Uuid;

/// Authenticated user identity, extracted from headers injected by Traefik ForwardAuth.
///
/// Traefik calls `/internal/verify` on auth-service before forwarding the request.
/// On success, it injects:
///   `X-User-Id`    → UUID string
///   `X-User-Role`  → "user" | "moderator" | "admin"
///   `X-User-Email` → email address
///
/// This extractor reads those headers directly — no JWT verification is needed
/// in downstream services.
pub struct RequestUser {
    pub id: Uuid,
    pub role: SystemRole,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let id = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok())
            .ok_or(AuthError::Unauthorized)?;

        let role = parts
            .headers
            .get("x-user-role")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| SystemRole::from_str(v).ok())
            .unwrap_or_default();

        let email = parts
            .headers
            .get("x-user-email")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // Surface the authenticated user on the active `http_request` span so
        // every log line emitted within this request carries `user_id`.
        tracing::Span::current().record("user_id", tracing::field::display(&id));

        Ok(RequestUser { id, role, email })
    }
}

#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    InsufficientPermissions,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::InsufficientPermissions => {
                (StatusCode::FORBIDDEN, "Insufficient permissions")
            }
        };
        (status, message).into_response()
    }
}
