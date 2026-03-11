#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use axum::http::{Method, StatusCode};
use common::helpers::{make_authenticated_request, make_json_request};
use common::setup::TestHarness;
use serde_json::json;
use uuid::Uuid;

// ============================================================
// GUILD MANAGEMENT
// ============================================================

#[tokio::test]
async fn create_guild_returns_201() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Test Guild", "description": "A test guild" })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Test Guild");
    assert_eq!(body["description"], "A test guild");
    assert_eq!(body["owner_id"], owner_id.to_string());
    assert_eq!(body["visibility"], "private");
    assert!(body["guild_id"].is_string());
}

#[tokio::test]
async fn create_guild_without_auth_returns_401() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router,
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Test Guild" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_guild_name_too_short_returns_422() {
    let harness = TestHarness::new().await;
    let token = harness.token_for(Uuid::new_v4());

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "X" })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
}

#[tokio::test]
async fn get_guild_returns_200() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    // Create a guild first
    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Visible Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["guild_id"], guild_id);
    assert_eq!(body["name"], "Visible Guild");
}

#[tokio::test]
async fn get_guild_not_found_returns_404() {
    let harness = TestHarness::new().await;
    let token = harness.token_for(Uuid::new_v4());
    let nonexistent = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{nonexistent}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn update_guild_by_owner_returns_200() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Old Name" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/guilds/{guild_id}"),
        Some(json!({
            "name": "New Name",
            "description": "Updated description"
        })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "New Name");
    assert_eq!(body["description"], "Updated description");
}

#[tokio::test]
async fn update_guild_by_non_owner_returns_403() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let other_token = harness.token_for(Uuid::new_v4());

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Protected Guild" })),
        &owner_token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/guilds/{guild_id}"),
        Some(json!({ "name": "Hijacked Name" })),
        &other_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn delete_guild_by_owner_returns_204() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Doomed Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone
    let (get_status, _) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}"),
        None,
        &token,
    )
    .await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_guild_by_non_owner_returns_403() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let other_token = harness.token_for(Uuid::new_v4());

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Safe Guild" })),
        &owner_token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}"),
        None,
        &other_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn search_guilds_returns_matching_results() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    // Create two guilds with distinct names
    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Rust Developers" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    // Make it public so it appears in search
    make_authenticated_request(
        harness.router.clone(),
        Method::PATCH,
        &format!("/api/v1/guilds/{guild_id}"),
        Some(json!({ "visibility": "public" })),
        &token,
    )
    .await;

    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Python Devs" })),
        &token,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        "/api/v1/guilds/search?query=Rust",
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let guilds = body["guilds"].as_array().unwrap();
    assert_eq!(guilds.len(), 1);
    assert_eq!(guilds[0]["name"], "Rust Developers");
}

// ============================================================
// GUILD ROLES
// ============================================================

#[tokio::test]
async fn list_roles_includes_default_everyone_role() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Role Test Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let roles = body.as_array().unwrap();
    assert_eq!(roles.len(), 1, "should have exactly the @everyone role");
    assert_eq!(roles[0]["is_default"], true);
    assert_eq!(roles[0]["name"], "@everyone");
}

#[tokio::test]
async fn create_role_by_owner_returns_201() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Guild With Roles" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Moderator", "permissions": 0 })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Moderator");
    assert_eq!(body["is_default"], false);
    assert_eq!(body["guild_id"], guild_id);
}

#[tokio::test]
async fn create_role_by_non_owner_returns_403() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let other_token = harness.token_for(Uuid::new_v4());

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Owner Only" })),
        &owner_token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Hacker Role", "permissions": 0 })),
        &other_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn get_role_returns_200() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Role Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (_, role_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Admin", "permissions": 8 })),
        &token,
    )
    .await;
    let role_id = role_body["role_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/roles/{role_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["role_id"], role_id);
    assert_eq!(body["name"], "Admin");
}

#[tokio::test]
async fn update_role_by_owner_returns_200() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (_, role_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Mod", "permissions": 0 })),
        &token,
    )
    .await;
    let role_id = role_body["role_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/guilds/{guild_id}/roles/{role_id}"),
        Some(json!({ "name": "Senior Mod", "hoist": true })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Senior Mod");
    assert_eq!(body["hoist"], true);
}

#[tokio::test]
async fn delete_custom_role_by_owner_returns_204() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (_, role_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Temp Role", "permissions": 0 })),
        &token,
    )
    .await;
    let role_id = role_body["role_id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/roles/{role_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_default_role_returns_400() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    // Fetch the @everyone role
    let (_, roles) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        None,
        &token,
    )
    .await;
    let everyone_id = roles
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["is_default"] == true)
        .unwrap()["role_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/roles/{everyone_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"].as_str().unwrap().contains("@everyone"));
}

// ============================================================
// GUILD MEMBERS
// ============================================================

#[tokio::test]
async fn list_members_returns_empty_after_create() {
    // create_guild does NOT auto-add the owner; member_count starts at 0
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Empty Guild" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/members"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);
    assert_eq!(body["members"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_members_returns_seeded_member() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Seeded Guild" })),
        &token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/members"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(
        body["members"][0]["user_id"].as_str().unwrap(),
        member_id.to_string()
    );
}

#[tokio::test]
async fn get_member_returns_200() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Get Member Guild" })),
        &token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/members/{member_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"].as_str().unwrap(), member_id.to_string());
    assert_eq!(body["guild_id"].as_str().unwrap(), guild_id.to_string());
}

#[tokio::test]
async fn get_member_not_found_returns_404() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Empty" })),
        &token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();
    let ghost_id = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/members/{ghost_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn kick_member_by_non_owner_returns_403() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let member_token = harness.token_for(member_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Kick Test" })),
        &owner_token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    // Seed a third user for the member to try kicking
    let victim_id = Uuid::new_v4();
    harness.seed_member(guild_id, victim_id).await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/members/{victim_id}"),
        Some(json!({})),
        &member_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn kick_member_by_owner_returns_204() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Kick Guild" })),
        &token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, victim_id).await;

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/members/{victim_id}"),
        Some(json!({})),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn owner_cannot_leave_guild() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "No Leave" })),
        &token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, owner_id).await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/members/@me"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn member_can_leave_guild() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let member_token = harness.token_for(member_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Leave Guild" })),
        &owner_token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/members/@me"),
        None,
        &member_token,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ============================================================
// GUILD INVITES
// ============================================================

/// Helper that creates a guild and seeds the owner as a member,
/// returning (`guild_id_string`, `owner_token`).
async fn create_guild_with_member_owner(harness: &TestHarness) -> (String, String) {
    let owner_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Invite Guild" })),
        &token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap().to_owned();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, owner_id).await;

    (guild_id_str, token)
}

#[tokio::test]
async fn create_invite_by_member_returns_201() {
    let harness = TestHarness::new().await;
    let (guild_id, token) = create_guild_with_member_owner(&harness).await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({ "max_uses": 10 })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["code"].as_str().unwrap().len() == 8);
    assert_eq!(body["guild_id"], guild_id);
    assert_eq!(body["max_uses"], 10);
    assert_eq!(body["uses"], 0);
}

#[tokio::test]
async fn create_invite_by_non_member_returns_403() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let outsider_token = harness.token_for(Uuid::new_v4());

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Members Only" })),
        &owner_token,
    )
    .await;
    let guild_id = created["guild_id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({})),
        &outsider_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn get_invite_is_public_no_auth_needed() {
    let harness = TestHarness::new().await;
    let (guild_id, token) = create_guild_with_member_owner(&harness).await;

    let (_, invite_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({})),
        &token,
    )
    .await;
    let code = invite_body["code"].as_str().unwrap();

    // Fetch without any Authorization header
    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/invites/{code}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["code"], code);
    assert_eq!(body["guild_id"], guild_id);
}

#[tokio::test]
async fn get_invite_not_found_returns_404() {
    let harness = TestHarness::new().await;

    let (status, body) = make_json_request(
        harness.router,
        Method::GET,
        "/api/v1/invites/ZZZZZZZZ",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn use_invite_joins_guild() {
    let harness = TestHarness::new().await;
    let (guild_id, owner_token) = create_guild_with_member_owner(&harness).await;

    let (_, invite_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({})),
        &owner_token,
    )
    .await;
    let code = invite_body["code"].as_str().unwrap();

    let joiner_id = Uuid::new_v4();
    let joiner_token = harness.token_for(joiner_id);

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/invites/{code}/use"),
        None,
        &joiner_token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], joiner_id.to_string());
    assert_eq!(body["guild_id"], guild_id);
}

#[tokio::test]
async fn use_invite_twice_returns_conflict() {
    let harness = TestHarness::new().await;
    let (guild_id, owner_token) = create_guild_with_member_owner(&harness).await;

    let (_, invite_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({})),
        &owner_token,
    )
    .await;
    let code = invite_body["code"].as_str().unwrap();

    let joiner_id = Uuid::new_v4();
    let joiner_token = harness.token_for(joiner_id);

    // First use - success
    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/invites/{code}/use"),
        None,
        &joiner_token,
    )
    .await;

    // Second use - already a member
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/invites/{code}/use"),
        None,
        &joiner_token,
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "conflict");
}

#[tokio::test]
async fn invite_exhausted_after_max_uses_returns_400() {
    let harness = TestHarness::new().await;
    let (guild_id, owner_token) = create_guild_with_member_owner(&harness).await;

    // Create invite with max_uses = 1
    let (_, invite_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({ "max_uses": 1 })),
        &owner_token,
    )
    .await;
    let code = invite_body["code"].as_str().unwrap();

    // First user consumes the only slot
    let first_user = Uuid::new_v4();
    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/invites/{code}/use"),
        None,
        &harness.token_for(first_user),
    )
    .await;

    // Second user should get 400 (invite exhausted)
    let second_user = Uuid::new_v4();
    let second_token = harness.token_for(second_user);
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/invites/{code}/use"),
        None,
        &second_token,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "bad_request");
}

#[tokio::test]
async fn revoke_invite_by_owner_returns_204() {
    let harness = TestHarness::new().await;
    let (guild_id, owner_token) = create_guild_with_member_owner(&harness).await;

    let (_, invite_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({})),
        &owner_token,
    )
    .await;
    let code = invite_body["code"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/invites/{code}"),
        None,
        &owner_token,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn list_invites_by_owner_returns_active_invites() {
    let harness = TestHarness::new().await;
    let (guild_id, owner_token) = create_guild_with_member_owner(&harness).await;

    // Create two invites
    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({})),
        &owner_token,
    )
    .await;
    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        Some(json!({ "max_uses": 5 })),
        &owner_token,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        None,
        &owner_token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let invites = body.as_array().unwrap();
    assert_eq!(invites.len(), 2);
}

#[tokio::test]
async fn list_invites_by_non_owner_returns_403() {
    let harness = TestHarness::new().await;
    let (guild_id, _) = create_guild_with_member_owner(&harness).await;
    let outsider_token = harness.token_for(Uuid::new_v4());

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/guilds/{guild_id}/invites"),
        None,
        &outsider_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

// ============================================================
// GET /api/v1/guilds/@me
// ============================================================

#[tokio::test]
async fn get_my_guilds_empty_when_not_a_member() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4();
    let token = harness.token_for(user_id);

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        "/api/v1/guilds/@me",
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);
    assert_eq!(body["guilds"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_my_guilds_returns_seeded_memberships() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let member_token = harness.token_for(member_id);

    // Create two guilds and seed member into both
    let (_, g1) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Guild Alpha" })),
        &owner_token,
    )
    .await;
    let guild1_id: Uuid = g1["guild_id"].as_str().unwrap().parse().unwrap();

    let (_, g2) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Guild Beta" })),
        &owner_token,
    )
    .await;
    let guild2_id: Uuid = g2["guild_id"].as_str().unwrap().parse().unwrap();

    harness.seed_member(guild1_id, member_id).await;
    harness.seed_member(guild2_id, member_id).await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        "/api/v1/guilds/@me",
        None,
        &member_token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 2);
    let guild_ids: Vec<&str> = body["guilds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["guild_id"].as_str().unwrap())
        .collect();
    assert!(guild_ids.contains(&guild1_id.to_string().as_str()));
    assert!(guild_ids.contains(&guild2_id.to_string().as_str()));
}

#[tokio::test]
async fn get_my_guilds_without_auth_returns_401() {
    let harness = TestHarness::new().await;

    let (status, _) =
        make_json_request(harness.router, Method::GET, "/api/v1/guilds/@me", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ============================================================
// ROLE ASSIGNMENT / REMOVAL
// ============================================================

#[tokio::test]
async fn assign_role_by_owner_returns_200() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    // Create guild and seed member
    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Role Assign Guild" })),
        &token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    // Create a custom role
    let (_, role_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Moderator", "permissions": 0 })),
        &token,
    )
    .await;
    let role_id = role_body["role_id"].as_str().unwrap();

    // Assign the role
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/guilds/{guild_id}/members/{member_id}/roles"),
        Some(json!({ "role_id": role_id })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], member_id.to_string());
    let role_ids = body["role_ids"].as_array().unwrap();
    assert!(
        role_ids.iter().any(|r| r.as_str() == Some(role_id)),
        "role_ids should contain the assigned role"
    );
}

#[tokio::test]
async fn assign_role_by_non_owner_returns_403() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let member_token = harness.token_for(member_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Forbidden Assign" })),
        &owner_token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    let (_, role_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "SomeRole", "permissions": 0 })),
        &owner_token,
    )
    .await;
    let role_id = role_body["role_id"].as_str().unwrap();

    // Member tries to assign role to themselves
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/guilds/{guild_id}/members/{member_id}/roles"),
        Some(json!({ "role_id": role_id })),
        &member_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn remove_role_by_owner_returns_200() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = harness.token_for(owner_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Role Remove Guild" })),
        &token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    // Create and assign a role
    let (_, role_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Temp Role", "permissions": 0 })),
        &token,
    )
    .await;
    let role_id = role_body["role_id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/guilds/{guild_id}/members/{member_id}/roles"),
        Some(json!({ "role_id": role_id })),
        &token,
    )
    .await;

    // Remove the role
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/members/{member_id}/roles/{role_id}"),
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], member_id.to_string());
    let role_ids = body["role_ids"].as_array().unwrap();
    assert!(
        !role_ids.iter().any(|r| r.as_str() == Some(role_id)),
        "role_ids should no longer contain the removed role"
    );
}

#[tokio::test]
async fn remove_role_by_non_owner_returns_403() {
    let harness = TestHarness::new().await;
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let owner_token = harness.token_for(owner_id);
    let member_token = harness.token_for(member_id);

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/guilds",
        Some(json!({ "name": "Remove Forbidden" })),
        &owner_token,
    )
    .await;
    let guild_id_str = created["guild_id"].as_str().unwrap();
    let guild_id: Uuid = guild_id_str.parse().unwrap();

    harness.seed_member(guild_id, member_id).await;

    let (_, role_body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/guilds/{guild_id}/roles"),
        Some(json!({ "name": "Target Role", "permissions": 0 })),
        &owner_token,
    )
    .await;
    let role_id = role_body["role_id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/guilds/{guild_id}/members/{member_id}/roles"),
        Some(json!({ "role_id": role_id })),
        &owner_token,
    )
    .await;

    // Member tries to remove their own role
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/guilds/{guild_id}/members/{member_id}/roles/{role_id}"),
        None,
        &member_token,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}
