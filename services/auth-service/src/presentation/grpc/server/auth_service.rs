use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::domain::auth_credential::AuthCredentialRepository;
use crate::domain::auth_session::AuthSessionRepository;
use crate::presentation::grpc::proto::auth::v1::auth_service_server::AuthService;
use crate::presentation::grpc::proto::auth::v1::{
    CheckPermissionRequest, CheckPermissionResponse, GetUserPermissionsRequest,
    GetUserPermissionsResponse, GetUserSessionsRequest, GetUserSessionsResponse,
    IntrospectTokenRequest, IntrospectTokenResponse, RevokeAllUserSessionsRequest,
    RevokeAllUserSessionsResponse, RevokeSessionRequest, SessionInfo, TokenClaims,
    ValidateTokenRequest, ValidateTokenResponse,
};
use common::infrastructure::security::jwt_manager::JwtManager;
 // Add EmailService import

/// gRPC server implementation for AuthService
///
/// Handles service-to-service authentication requests:
/// - Token validation for other services
/// - Session management
/// - Permission checks
pub struct AuthServiceGrpc {
    jwt_manager: Arc<JwtManager>,
    #[allow(dead_code)]
    credential_repo: Arc<dyn AuthCredentialRepository>,
    session_repo: Arc<dyn AuthSessionRepository>,
}

impl AuthServiceGrpc {
    pub fn new(
        jwt_manager: Arc<JwtManager>,
        credential_repo: Arc<dyn AuthCredentialRepository>,
        session_repo: Arc<dyn AuthSessionRepository>,
    ) -> Self {
        Self {
            jwt_manager,
            credential_repo,
            session_repo,
        }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceGrpc {
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.jwt_manager.verify_access_token(&req.access_token) {
            Ok(claims) => {
                let expires_at = prost_types::Timestamp {
                    seconds: claims.exp,
                    nanos: 0,
                };
                let issued_at = prost_types::Timestamp {
                    seconds: claims.iat,
                    nanos: 0,
                };

                Ok(Response::new(ValidateTokenResponse {
                    valid: true,
                    claims: Some(TokenClaims {
                        user_id: claims.sub,
                        email: claims.email,
                        role: String::new(), // Role not yet in JWT claims
                        expires_at: Some(expires_at),
                        issued_at: Some(issued_at),
                    }),
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                claims: None,
                error: e.to_string(),
            })),
        }
    }

    async fn introspect_token(
        &self,
        request: Request<IntrospectTokenRequest>,
    ) -> Result<Response<IntrospectTokenResponse>, Status> {
        let req = request.into_inner();

        // Try access token first, then refresh token
        let claims = if req.token_type_hint == "refresh_token" {
            self.jwt_manager.verify_refresh_token(&req.token).ok()
        } else {
            self.jwt_manager
                .verify_access_token(&req.token)
                .ok()
                .or_else(|| self.jwt_manager.verify_refresh_token(&req.token).ok())
        };

        match claims {
            Some(c) => Ok(Response::new(IntrospectTokenResponse {
                active: c.is_valid(),
                scope: String::new(),
                client_id: c.aud,
                username: c.email,
                exp: c.exp,
                iat: c.iat,
            })),
            None => Ok(Response::new(IntrospectTokenResponse {
                active: false,
                scope: String::new(),
                client_id: String::new(),
                username: String::new(),
                exp: 0,
                iat: 0,
            })),
        }
    }

    async fn get_user_sessions(
        &self,
        request: Request<GetUserSessionsRequest>,
    ) -> Result<Response<GetUserSessionsResponse>, Status> {
        let req = request.into_inner();

        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        let credential = self
            .credential_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Credential not found"))?;

        let sessions = self
            .session_repo
            .find_active_by_credential_id(credential.id())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let session_infos: Vec<SessionInfo> = sessions
            .iter()
            .map(|s| SessionInfo {
                session_id: s.id().to_string(),
                device_name: s.device_name().unwrap_or_default().to_string(),
                ip_address: s.ip_address().unwrap_or_default().to_string(),
                user_agent: s.user_agent().unwrap_or_default().to_string(),
                created_at: Some(prost_types::Timestamp {
                    seconds: s.created_at().timestamp(),
                    nanos: 0,
                }),
                last_used_at: Some(prost_types::Timestamp {
                    seconds: s.last_used_at().timestamp(),
                    nanos: 0,
                }),
                expires_at: Some(prost_types::Timestamp {
                    seconds: s.expires_at().timestamp(),
                    nanos: 0,
                }),
                is_current: false, // Cannot determine without current session context
            })
            .collect();

        Ok(Response::new(GetUserSessionsResponse {
            sessions: session_infos,
        }))
    }

    async fn revoke_session(
        &self,
        request: Request<RevokeSessionRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|_| Status::invalid_argument("Invalid session_id format"))?;

        let _user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        let mut session = self
            .session_repo
            .find_by_id(session_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Session not found"))?;

        session.revoke();

        self.session_repo
            .update(&session)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn revoke_all_user_sessions(
        &self,
        request: Request<RevokeAllUserSessionsRequest>,
    ) -> Result<Response<RevokeAllUserSessionsResponse>, Status> {
        let req = request.into_inner();

        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        let credential = self
            .credential_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Credential not found"))?;

        // If except_session_id is provided, we need to handle it differently
        let revoked_count = if let Some(except_id) = &req.except_session_id {
            let _except_uuid = Uuid::parse_str(except_id)
                .map_err(|_| Status::invalid_argument("Invalid except_session_id format"))?;

            // Get all active sessions and revoke individually (except the excluded one)
            let sessions = self
                .session_repo
                .find_active_by_credential_id(credential.id())
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            let mut count = 0;
            for mut session in sessions {
                if session.id().to_string() != *except_id {
                    session.revoke();
                    self.session_repo
                        .update(&session)
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?;
                    count += 1;
                }
            }
            count
        } else {
            self.session_repo
                .revoke_all_by_credential_id(credential.id())
                .await
                .map_err(|e| Status::internal(e.to_string()))? as i32
        };

        Ok(Response::new(RevokeAllUserSessionsResponse {
            sessions_revoked: revoked_count,
        }))
    }

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();

        let _user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        // TODO: Implement actual permission checking logic
        // For now, allow all authenticated users
        Ok(Response::new(CheckPermissionResponse {
            allowed: true,
            reason: String::new(),
        }))
    }

    async fn get_user_permissions(
        &self,
        request: Request<GetUserPermissionsRequest>,
    ) -> Result<Response<GetUserPermissionsResponse>, Status> {
        let req = request.into_inner();

        let _user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id format"))?;

        // TODO: Implement actual permission retrieval logic
        // For now, return default user permissions
        Ok(Response::new(GetUserPermissionsResponse {
            permissions: vec![
                "profile.read".to_string(),
                "profile.update".to_string(),
                "channels.read".to_string(),
                "messages.send".to_string(),
            ],
            role: "user".to_string(),
        }))
    }
}
