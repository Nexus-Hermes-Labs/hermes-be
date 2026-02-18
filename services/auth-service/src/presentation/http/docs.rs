use utoipa::OpenApi;

use crate::presentation::http::dto::*;
use crate::presentation::http::handlers::auth::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        register_handler,
        login_handler,
        refresh_token_handler,
        logout_handler,
        verify_email_handler
    ),
    components(
        schemas(
            LoginRequest, AuthResponse, LogoutRequest, LogoutResponse,
            RefreshTokenRequest, RegisterRequest, AuthResponseWithUser,
            UserProfile, VerifyEmailResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication and Session Management API")
    )
)]
pub struct ApiDoc;
