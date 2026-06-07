use utoipa::OpenApi;

use crate::presentation::http::dto::*;
use crate::presentation::http::handlers::auth::*;
use crate::presentation::http::handlers::oauth::*;
use common::observability::health::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        register_handler,
        login_handler,
        refresh_token_handler,
        logout_handler,
        verify_email_handler,
        google_authorize_handler,
        google_callback_handler,
        health_handler,
        liveness_handler,
        readiness_handler
    ),
    components(
        schemas(
            LoginRequest, AuthResponse, LogoutRequest, LogoutResponse,
            RefreshTokenRequest, RegisterRequest, AuthResponseWithUser,
            UserProfile, VerifyEmailResponse,
            AuthorizeUrlResponse, GoogleCallbackRequest,
            HealthResponse, HealthChecks, ComponentHealth
        )
    ),
    tags(
        (name = "auth", description = "Authentication and Session Management API"),
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;
