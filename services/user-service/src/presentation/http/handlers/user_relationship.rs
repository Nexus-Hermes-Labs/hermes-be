use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::state::AppState;
use crate::presentation::dto::user_relationship::{RelationshipRequest, RelationshipResponse, Pagination};

// ============================================
// Handler
// ============================================

pub struct UserRelationshipHandler;

impl UserRelationshipHandler {
    // POST /api/users/users/{user_id}/relationships/request
    pub async fn send_friend_request(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Json(payload): Json<RelationshipRequest>,
    ) -> Result<Json<RelationshipResponse>, StatusCode> {
        let relationship = state
            .user
            .relationship_service
            .send_friend_request(user_id, payload.target_user_id, payload.message)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(RelationshipResponse {
            user_id: relationship.user_id(),
            target_user_id: relationship.target_user_id(),
            r#type: relationship.relationship_type().to_string(),
            message: relationship.message().map(|s| s.to_string()),
        }))
    }

    // PUT /api/users/users/{user_id}/relationships/request/accept
    pub async fn accept_friend_request(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Json(payload): Json<RelationshipRequest>,
    ) -> Result<Json<RelationshipResponse>, StatusCode> {
        let relationship = state
            .user
            .relationship_service
            .accept_friend_request(user_id, payload.target_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(RelationshipResponse {
            user_id: relationship.user_id(),
            target_user_id: relationship.target_user_id(),
            r#type: relationship.relationship_type().to_string(),
            message: relationship.message().map(|s| s.to_string()),
        }))
    }

    // PUT /api/users/users/{user_id}/relationships/request/decline
    pub async fn decline_friend_request(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Json(payload): Json<RelationshipRequest>,
    ) -> Result<StatusCode, StatusCode> {
        state
            .user
            .relationship_service
            .decline_friend_request(user_id, payload.target_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::NO_CONTENT)
    }

    // DELETE /api/users/users/{user_id}/relationships/friend/{target_user_id}
    pub async fn remove_friend(
        State(state): State<AppState>,
        Path((user_id, target_user_id)): Path<(Uuid, Uuid)>,
    ) -> Result<StatusCode, StatusCode> {
        state
            .user
            .relationship_service
            .remove_friend(user_id, target_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::NO_CONTENT)
    }

    // POST /api/users/users/{user_id}/relationships/block
    pub async fn block_user(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Json(payload): Json<RelationshipRequest>,
    ) -> Result<Json<RelationshipResponse>, StatusCode> {
        let relationship = state
            .user
            .relationship_service
            .block_user(user_id, payload.target_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(RelationshipResponse {
            user_id: relationship.user_id(),
            target_user_id: relationship.target_user_id(),
            r#type: relationship.relationship_type().to_string(),
            message: relationship.message().map(|s| s.to_string()),
        }))
    }

    // DELETE /api/users/users/{user_id}/relationships/block/{target_user_id}
    pub async fn unblock_user(
        State(state): State<AppState>,
        Path((user_id, target_user_id)): Path<(Uuid, Uuid)>,
    ) -> Result<StatusCode, StatusCode> {
        state
            .user
            .relationship_service
            .unblock_user(user_id, target_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::NO_CONTENT)
    }

    // GET /api/users/users/{user_id}/relationships/{target_user_id}
    pub async fn get_relationship(
        State(state): State<AppState>,
        Path((user_id, target_user_id)): Path<(Uuid, Uuid)>,
    ) -> Result<Json<RelationshipResponse>, StatusCode> {
        let relationship = state
            .user
            .relationship_service
            .get_relationship(user_id, target_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        Ok(Json(RelationshipResponse {
            user_id: relationship.user_id(),
            target_user_id: relationship.target_user_id(),
            r#type: relationship.relationship_type().to_string(),
            message: relationship.message().map(|s| s.to_string()),
        }))
    }

    // GET /api/users/users/{user_id}/relationships/friends
    pub async fn get_friends(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Query(pagination): Query<Pagination>,
    ) -> Result<Json<Vec<RelationshipResponse>>, StatusCode> {
        let relationships = state
            .user
            .relationship_service
            .get_friends(user_id, pagination.limit, pagination.offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let response = relationships
            .iter()
            .map(|r| RelationshipResponse {
                user_id: r.user_id(),
                target_user_id: r.target_user_id(),
                r#type: r.relationship_type().to_string(),
                message: r.message().map(|s| s.to_string()),
            })
            .collect();

        Ok(Json(response))
    }

    // GET /api/users/users/{user_id}/relationships/incoming
    pub async fn get_incoming_requests(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Query(pagination): Query<Pagination>,
    ) -> Result<Json<Vec<RelationshipResponse>>, StatusCode> {
        let relationships = state
            .user
            .relationship_service
            .get_incoming_requests(user_id, pagination.limit, pagination.offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let response = relationships
            .iter()
            .map(|r| RelationshipResponse {
                user_id: r.user_id(),
                target_user_id: r.target_user_id(),
                r#type: r.relationship_type().to_string(),
                message: r.message().map(|s| s.to_string()),
            })
            .collect();

        Ok(Json(response))
    }

    // GET /api/users/users/{user_id}/relationships/outgoing
    pub async fn get_outgoing_requests(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Query(pagination): Query<Pagination>,
    ) -> Result<Json<Vec<RelationshipResponse>>, StatusCode> {
        let relationships = state
            .user
            .relationship_service
            .get_outgoing_requests(user_id, pagination.limit, pagination.offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let response = relationships
            .iter()
            .map(|r| RelationshipResponse {
                user_id: r.user_id(),
                target_user_id: r.target_user_id(),
                r#type: r.relationship_type().to_string(),
                message: r.message().map(|s| s.to_string()),
            })
            .collect();

        Ok(Json(response))
    }

    // GET /api/users/users/{user_id}/relationships/blocked
    pub async fn get_blocked_users(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
        Query(pagination): Query<Pagination>,
    ) -> Result<Json<Vec<RelationshipResponse>>, StatusCode> {
        let relationships = state
            .user
            .relationship_service
            .get_blocked_users(user_id, pagination.limit, pagination.offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let response = relationships
            .iter()
            .map(|r| RelationshipResponse {
                user_id: r.user_id(),
                target_user_id: r.target_user_id(),
                r#type: r.relationship_type().to_string(),
                message: r.message().map(|s| s.to_string()),
            })
            .collect();

        Ok(Json(response))
    }
}
