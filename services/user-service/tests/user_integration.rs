mod common;

use axum::http::{Method, StatusCode};
use common::helpers::{make_authenticated_request, make_json_request};
use common::setup::TestHarness;
use serde_json::json;

// ============================================
// HELPER: create a profile and return (status, body)
// ============================================
async fn create_profile(
    harness: &TestHarness,
    username: &str,
    display_name: &str,
) -> (StatusCode, serde_json::Value) {
    make_json_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users",
        Some(json!({
            "username": username,
            "display_name": display_name
        })),
    )
    .await
}

/// Creates a profile and returns (user_id, user_id).
///
/// The second value was previously a JWT token; now authentication uses the
/// Traefik-injected `X-User-Id` header, so the user_id itself is passed to
/// `make_authenticated_request` directly.
async fn create_profile_with_token(
    harness: &TestHarness,
    username: &str,
    display_name: &str,
) -> (String, String) {
    let (status, body) = create_profile(harness, username, display_name).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create {username} failed: {body}"
    );
    let user_id = body["user_id"].as_str().expect("user_id").to_string();
    (user_id.clone(), user_id)
}

// ============================================
// SYSTEM-LEVEL PROFILE MANAGEMENT TESTS
// ============================================

#[tokio::test]
async fn test_create_profile_success() {
    let harness = TestHarness::new().await;

    let (status, body) = create_profile(&harness, "alice", "Alice").await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["username"], "alice");
    assert_eq!(body["display_name"], "Alice");
    assert!(body["user_id"].is_string());
}

#[tokio::test]
async fn test_create_profile_invalid_username() {
    let harness = TestHarness::new().await;

    // Uppercase not allowed
    let (status, _) = create_profile(&harness, "Alice", "Alice").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Too short (< 3 chars)
    let (status, _) = create_profile(&harness, "ab", "AB").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_create_profile_duplicate_username() {
    let harness = TestHarness::new().await;

    let (status, _) = create_profile(&harness, "dupuser", "Dup One").await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = create_profile(&harness, "dupuser", "Dup Two").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// ============================================
// GET PROFILE TESTS
// ============================================

#[tokio::test]
async fn test_get_profile_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "getme", "Get Me").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "getme");
    assert_eq!(body["display_name"], "Get Me");
}

#[tokio::test]
async fn test_get_profile_not_found() {
    let harness = TestHarness::new().await;
    let random_id = uuid::Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{random_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_profile_by_username() {
    let harness = TestHarness::new().await;

    create_profile(&harness, "byname", "By Name").await;

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/username/byname",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "byname");
}

// ============================================
// UPDATE PROFILE TESTS
// ============================================

#[tokio::test]
async fn test_update_profile_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "updateme", "Update Me").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PATCH,
        &format!("/api/v1/users/{user_id}"),
        Some(json!({
            "display_name": "Updated Name",
            "bio": "A new bio"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["display_name"], "Updated Name");
    assert_eq!(body["bio"], "A new bio");
}

#[tokio::test]
async fn test_update_profile_not_found() {
    let harness = TestHarness::new().await;
    let random_id = uuid::Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PATCH,
        &format!("/api/v1/users/{random_id}"),
        Some(json!({ "display_name": "Ghost" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================
// SYSTEM-LEVEL CHANGE USERNAME TESTS
// ============================================

#[tokio::test]
async fn test_change_username_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "oldname", "Old Name").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_id}/username"),
        Some(json!({ "new_username": "newname" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "newname");
}

#[tokio::test]
async fn test_change_username_already_taken() {
    let harness = TestHarness::new().await;

    create_profile(&harness, "taken_name", "Taken").await;
    let (_, create_body) = create_profile(&harness, "wants_taken", "Wants Taken").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_id}/username"),
        Some(json!({ "new_username": "taken_name" })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

// ============================================
// SYSTEM-LEVEL DELETE PROFILE TESTS
// ============================================

#[tokio::test]
async fn test_delete_profile_success() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "deleteme", "Delete Me").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/{user_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_then_get() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "delete_get", "Delete Get").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    // Delete
    make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/{user_id}"),
        None,
    )
    .await;

    // Get should now 404
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================
// SYSTEM-LEVEL STATUS & PRESENCE TESTS
// ============================================

#[tokio::test]
async fn test_update_status() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "statususer", "Status User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    for status_val in &["online", "idle", "dnd", "offline"] {
        let (status, body) = make_json_request(
            harness.router.clone(),
            Method::PUT,
            &format!("/api/v1/users/{user_id}/status"),
            Some(json!({ "status": status_val })),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "setting {status_val} failed: {body}"
        );
        assert_eq!(body["status"], *status_val);
    }
}

#[tokio::test]
async fn test_set_custom_status() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "customstatus", "Custom Status").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_id}/custom-status"),
        Some(json!({
            "text": "Working hard",
            "emoji": "hammer"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["custom_status"]["text"], "Working hard");
    assert_eq!(body["custom_status"]["emoji"], "hammer");
}

#[tokio::test]
async fn test_clear_custom_status() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "clearstatus", "Clear Status").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    // Set a custom status first
    make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_id}/custom-status"),
        Some(json!({ "text": "Temporary" })),
    )
    .await;

    // Clear it
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/{user_id}/custom-status"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ============================================
// SYSTEM-LEVEL SEARCH & DISCOVERY TESTS
// ============================================

#[tokio::test]
async fn test_check_username_availability() {
    let harness = TestHarness::new().await;

    // Available
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/check-username/available_name",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "available");

    // Take it
    create_profile(&harness, "available_name", "Taken Now").await;

    // No longer available
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/check-username/available_name",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "taken");
}

#[tokio::test]
async fn test_search_users() {
    let harness = TestHarness::new().await;

    create_profile(&harness, "searchable_alice", "Alice Searchable").await;
    create_profile(&harness, "searchable_bob", "Bob Searchable").await;

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/search?query=searchable&limit=10&offset=0",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    let profiles = body["profiles"].as_array().expect("profiles array");
    assert!(
        profiles.len() >= 2,
        "expected at least 2 results, got {}",
        profiles.len()
    );
}

// ============================================
// SYSTEM-LEVEL RELATIONSHIP TESTS
// ============================================

#[tokio::test]
async fn test_block_user_preserves_reverse_block() {
    let harness = TestHarness::new().await;

    // 1. Create two users
    let (_, user_a_body) = create_profile(&harness, "blocker_a", "Blocker A").await;
    let user_a_id = user_a_body["user_id"].as_str().unwrap();

    let (_, user_b_body) = create_profile(&harness, "blocker_b", "Blocker B").await;
    let user_b_id = user_b_body["user_id"].as_str().unwrap();

    // 2. User B blocks User A
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_b_id}/relationships/block"),
        Some(json!({ "target_user_id": user_a_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. User A blocks User B
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_a_id}/relationships/block"),
        Some(json!({ "target_user_id": user_b_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 4. Verify User B's block on User A still exists
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_b_id}/relationships/{user_a_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["type"], "blocked");
}

#[tokio::test]
async fn test_friend_request_accept_and_remove() {
    let harness = TestHarness::new().await;

    // 1. Create two users
    let (_, user_a_body) = create_profile(&harness, "friend_a", "Friend A").await;
    let user_a_id = user_a_body["user_id"].as_str().unwrap();

    let (_, user_b_body) = create_profile(&harness, "friend_b", "Friend B").await;
    let user_b_id = user_b_body["user_id"].as_str().unwrap();

    // 2. User A sends friend request to User B
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_a_id}/relationships/request"),
        Some(json!({ "target_user_id": user_b_id, "message": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. Verify pending status
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_a_id}/relationships/{user_b_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["type"], "pending_outgoing");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_b_id}/relationships/{user_a_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["type"], "pending_incoming");

    // 4. User B accepts the request
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_b_id}/relationships/request/accept"),
        Some(json!({ "target_user_id": user_a_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 5. Verify friendship
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_a_id}/relationships/{user_b_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["type"], "friend");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_b_id}/relationships/{user_a_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["type"], "friend");

    // 6. User A removes User B
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/{user_a_id}/relationships/friend/{user_b_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 7. Verify removal
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_a_id}/relationships/{user_b_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_b_id}/relationships/{user_a_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_friend_request_decline() {
    let harness = TestHarness::new().await;

    // 1. Create two users
    let (_, user_c_body) = create_profile(&harness, "friend_c", "Friend C").await;
    let user_c_id = user_c_body["user_id"].as_str().unwrap();

    let (_, user_d_body) = create_profile(&harness, "friend_d", "Friend D").await;
    let user_d_id = user_d_body["user_id"].as_str().unwrap();

    // 2. User C sends friend request to User D
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_c_id}/relationships/request"),
        Some(json!({ "target_user_id": user_d_id, "message": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. User D declines the request
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_d_id}/relationships/request/decline"),
        Some(json!({ "target_user_id": user_c_id })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 4. Verify removal
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_c_id}/relationships/{user_d_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_d_id}/relationships/{user_c_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_unblock_user() {
    let harness = TestHarness::new().await;

    // 1. Create two users
    let (_, user_c_body) = create_profile(&harness, "blocker_c", "Blocker C").await;
    let user_c_id = user_c_body["user_id"].as_str().unwrap();

    let (_, user_d_body) = create_profile(&harness, "blocker_d", "Blocker D").await;
    let user_d_id = user_d_body["user_id"].as_str().unwrap();

    // 2. User C blocks User D
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_c_id}/relationships/block"),
        Some(json!({ "target_user_id": user_d_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. User C unblocks User D
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/{user_c_id}/relationships/block/{user_d_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 4. Verify removal
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_c_id}/relationships/{user_d_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_friend_request_to_blocked_user() {
    let harness = TestHarness::new().await;

    // 1. Create two users
    let (_, user_e_body) = create_profile(&harness, "blocker_e", "Blocker E").await;
    let user_e_id = user_e_body["user_id"].as_str().unwrap();

    let (_, user_f_body) = create_profile(&harness, "blocker_f", "Blocker F").await;
    let user_f_id = user_f_body["user_id"].as_str().unwrap();

    // 2. User E blocks User F
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_e_id}/relationships/block"),
        Some(json!({ "target_user_id": user_f_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. User E (blocker) tries to send a friend request to User F (blocked) -> Fails
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_e_id}/relationships/request"),
        Some(json!({ "target_user_id": user_f_id, "message": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 4. User F (blocked) tries to send a friend request to User E (blocker) -> Fails
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_f_id}/relationships/request"),
        Some(json!({ "target_user_id": user_e_id, "message": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_friend_request_privacy_none() {
    let harness = TestHarness::new().await;

    // 1. Create two users
    let (_, user_a_body) = create_profile(&harness, "privacy_a", "Privacy A").await;
    let user_a_id = user_a_body["user_id"].as_str().unwrap();

    let (_, user_b_body) = create_profile(&harness, "privacy_b", "Privacy B").await;
    let user_b_id = user_b_body["user_id"].as_str().unwrap();

    // 2. User B sets friend request privacy to "none"
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_b_id}/privacy/friend-requests"),
        Some(json!({ "allow_friend_requests_from": "none" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. User A attempts to send a friend request to User B -> Fails
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_a_id}/relationships/request"),
        Some(json!({ "target_user_id": user_b_id, "message": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ============================================
// SYSTEM-LEVEL PRIVACY TESTS
// ============================================

#[tokio::test]
async fn test_get_privacy_defaults() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "privacyuser", "Privacy User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_id}/privacy"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    // Check defaults from the migration
    assert_eq!(body["allow_dms_from"], "friends");
    assert_eq!(body["allow_friend_requests_from"], "everyone");
    assert_eq!(body["show_online_status"], true);
    assert_eq!(body["show_current_activity"], true);
    assert_eq!(body["show_profile_to_non_friends"], true);
    assert_eq!(body["allow_nsfw_content"], false);
    assert_eq!(body["content_filter_level"], 1);
}

#[tokio::test]
async fn test_update_dm_privacy() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "dmuser", "DM User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_id}/privacy/dm"),
        Some(json!({ "allow_dms_from": "none" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_dms_from"], "none");
}

#[tokio::test]
async fn test_update_friend_request_privacy() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "fruser", "FR User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{user_id}/privacy/friend-requests"),
        Some(json!({ "allow_friend_requests_from": "none" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_friend_requests_from"], "none");
}

#[tokio::test]
async fn test_apply_privacy_preset() {
    let harness = TestHarness::new().await;

    let (_, create_body) = create_profile(&harness, "presetuser", "Preset User").await;
    let user_id = create_body["user_id"].as_str().expect("user_id");

    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{user_id}/privacy/preset"),
        Some(json!({ "preset": "private" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    // Private preset should restrict most settings
    assert_eq!(body["allow_dms_from"], "none");
    assert_eq!(body["allow_friend_requests_from"], "none");
    assert_eq!(body["show_online_status"], false);
    assert_eq!(body["show_current_activity"], false);
    assert_eq!(body["show_profile_to_non_friends"], false);
}

// ============================================
// SYSTEM-LEVEL COMPLEX INTERACTION TESTS
// ============================================

#[tokio::test]
async fn test_complex_privacy_and_relationship_flow() {
    let harness = TestHarness::new().await;

    // 1. Create 3 users: Alice, Bob, Charlie
    let (_, alice_body) = create_profile(&harness, "alice", "Alice").await;
    let alice_id = alice_body["user_id"].as_str().unwrap();

    let (_, bob_body) = create_profile(&harness, "bob", "Bob").await;
    let bob_id = bob_body["user_id"].as_str().unwrap();

    let (_, charlie_body) = create_profile(&harness, "charlie", "Charlie").await;
    let charlie_id = charlie_body["user_id"].as_str().unwrap();

    // 2. Alice and Bob become friends
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{alice_id}/relationships/request"),
        Some(json!({ "target_user_id": bob_id, "message": "Hey Bob!" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{bob_id}/relationships/request/accept"),
        Some(json!({ "target_user_id": alice_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. Alice sets friend request privacy to "none"
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/users/{alice_id}/privacy/friend-requests"),
        Some(json!({ "allow_friend_requests_from": "none" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 4. Charlie tries to send friend request to Alice -> Fails due to Alice's privacy
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{charlie_id}/relationships/request"),
        Some(json!({ "target_user_id": alice_id, "message": "Hi Alice" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 5. Alice blocks Charlie
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/users/{alice_id}/relationships/block"),
        Some(json!({ "target_user_id": charlie_id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 6. Charlie tries to get Alice's profile -> Should still work for basic info
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{alice_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 7. Verify Alice blocked Charlie
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{charlie_id}/relationships/{alice_id}"),
        None,
    )
    .await;
    // Since blocks are one-way, Charlie has no relationship record for Alice
    // TODO: we have think about that. should block should one way or both of them should see same in db?
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================
// @ME AUTHENTICATION TESTS
// ============================================

#[tokio::test]
async fn test_me_unauthenticated_returns_401() {
    let harness = TestHarness::new().await;

    // No token -> 401
    let (status, _) = make_json_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_me_invalid_token_returns_401() {
    let harness = TestHarness::new().await;

    // Non-UUID X-User-Id header cannot be parsed → RequestUser extractor returns 401
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me",
        None,
        "invalid.user.id",
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_me_expired_token_returns_401() {
    let harness = TestHarness::new().await;

    // Malformed (but non-empty) X-User-Id header → RequestUser extractor returns 401
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me",
        None,
        "not-a-valid-uuid",
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ============================================
// @ME PROFILE TESTS
// ============================================

#[tokio::test]
async fn test_me_get_profile() {
    let harness = TestHarness::new().await;
    let (_user_id, token) = create_profile_with_token(&harness, "meprofile", "Me Profile").await;

    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me",
        None,
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "meprofile");
    assert_eq!(body["display_name"], "Me Profile");
}

#[tokio::test]
async fn test_me_update_profile() {
    let harness = TestHarness::new().await;
    let (_user_id, token) = create_profile_with_token(&harness, "meupdater", "Me Updater").await;

    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::PATCH,
        "/api/v1/users/@me",
        Some(json!({
            "display_name": "Updated Me",
            "bio": "I updated myself"
        })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["display_name"], "Updated Me");
    assert_eq!(body["bio"], "I updated myself");
}

#[tokio::test]
async fn test_me_change_username() {
    let harness = TestHarness::new().await;
    let (_user_id, token) = create_profile_with_token(&harness, "meoldname", "Me Old Name").await;

    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/username",
        Some(json!({ "new_username": "menewname" })),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["username"], "menewname");
}

#[tokio::test]
async fn test_me_delete_then_get_returns_404() {
    let harness = TestHarness::new().await;
    let (_user_id, token) = create_profile_with_token(&harness, "medeleteme", "Me Delete Me").await;

    // Delete via @me
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        "/api/v1/users/@me",
        None,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Subsequent GET should 404
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me",
        None,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================
// @ME STATUS & CUSTOM STATUS TESTS
// ============================================

#[tokio::test]
async fn test_me_status_lifecycle() {
    let harness = TestHarness::new().await;
    let (_user_id, token) = create_profile_with_token(&harness, "mestatuser", "Me Stat User").await;

    // Cycle through all statuses
    for status_val in &["online", "idle", "dnd", "offline"] {
        let (status, body) = make_authenticated_request(
            harness.router.clone(),
            Method::PUT,
            "/api/v1/users/@me/status",
            Some(json!({ "status": status_val })),
            &token,
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "setting {status_val} failed: {body}"
        );
        assert_eq!(body["status"], *status_val);
    }

    // Verify final status is persisted
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me",
        None,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "offline");
}

#[tokio::test]
async fn test_me_custom_status_set_and_clear() {
    let harness = TestHarness::new().await;
    let (_user_id, token) = create_profile_with_token(&harness, "mecuststat", "Me Cust Stat").await;

    // Set custom status
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/custom-status",
        Some(json!({
            "text": "In a meeting",
            "emoji": "calendar"
        })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["custom_status"]["text"], "In a meeting");
    assert_eq!(body["custom_status"]["emoji"], "calendar");

    // Clear custom status
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        "/api/v1/users/@me/custom-status",
        None,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's cleared
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me",
        None,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["custom_status"].is_null());
}

// ============================================
// @ME PRIVACY TESTS
// ============================================

#[tokio::test]
async fn test_me_privacy_full_lifecycle() {
    let harness = TestHarness::new().await;
    let (_user_id, token) = create_profile_with_token(&harness, "meprivuser", "Me Priv User").await;

    // 1. Get defaults
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/privacy",
        None,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_dms_from"], "friends");
    assert_eq!(body["allow_friend_requests_from"], "everyone");

    // 2. Update DM privacy
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/privacy/dm",
        Some(json!({ "allow_dms_from": "none" })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_dms_from"], "none");

    // 3. Update friend request privacy
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/privacy/friend-requests",
        Some(json!({ "allow_friend_requests_from": "none" })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_friend_requests_from"], "none");

    // 4. Update visibility
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::PATCH,
        "/api/v1/users/@me/privacy/visibility",
        Some(json!({
            "show_online_status": false,
            "show_current_activity": false,
            "show_profile_to_non_friends": false
        })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["show_online_status"], false);
    assert_eq!(body["show_current_activity"], false);
    assert_eq!(body["show_profile_to_non_friends"], false);

    // 5. Update content settings
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::PATCH,
        "/api/v1/users/@me/privacy/content",
        Some(json!({
            "allow_nsfw_content": true,
            "content_filter_level": 2
        })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_nsfw_content"], true);
    assert_eq!(body["content_filter_level"], 2);

    // 6. Apply public preset to reset everything
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/privacy/preset",
        Some(json!({ "preset": "public" })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_dms_from"], "everyone");
    assert_eq!(body["allow_friend_requests_from"], "everyone");
    assert_eq!(body["show_online_status"], true);
}

// ============================================
// @ME RELATIONSHIP TESTS
// ============================================

#[tokio::test]
async fn test_me_friend_request_accept_and_remove() {
    let harness = TestHarness::new().await;

    let (alice_id, alice_token) = create_profile_with_token(&harness, "me_alice", "Me Alice").await;
    let (_bob_id, bob_token) = create_profile_with_token(&harness, "me_bob", "Me Bob").await;
    let bob_id_str = _bob_id.as_str();

    // 1. Alice sends friend request to Bob via @me
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": bob_id_str, "message": "Hey Bob!" })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "send request failed: {body}");

    // 2. Alice sees outgoing request
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/outgoing?limit=10&offset=0",
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let outgoing = body.as_array().expect("outgoing array");
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0]["target_user_id"], bob_id_str);

    // 3. Bob sees incoming request
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/incoming?limit=10&offset=0",
        None,
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let incoming = body.as_array().expect("incoming array");
    assert_eq!(incoming.len(), 1);

    // 4. Bob accepts via @me
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/relationships/request/accept",
        Some(json!({ "target_user_id": alice_id })),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 5. Both see each other in friends list
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/friends?limit=10&offset=0",
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let friends = body.as_array().expect("friends array");
    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0]["type"], "friend");

    // 6. Alice checks relationship with Bob
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/@me/relationships/{bob_id_str}"),
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["type"], "friend");

    // 7. Alice removes Bob as friend via @me
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/@me/relationships/friend/{bob_id_str}"),
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 8. Verify removal
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/@me/relationships/{bob_id_str}"),
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_me_friend_request_decline() {
    let harness = TestHarness::new().await;

    let (alice_id, alice_token) =
        create_profile_with_token(&harness, "me_decl_a", "Me Decl A").await;
    let (_bob_id, bob_token) = create_profile_with_token(&harness, "me_decl_b", "Me Decl B").await;
    let bob_id_str = _bob_id.as_str();

    // Alice sends request
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": bob_id_str, "message": "hi" })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Bob declines
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/relationships/request/decline",
        Some(json!({ "target_user_id": alice_id })),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify no relationship exists
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/@me/relationships/{bob_id_str}"),
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_me_block_and_unblock() {
    let harness = TestHarness::new().await;

    let (_alice_id, alice_token) =
        create_profile_with_token(&harness, "me_blk_a", "Me Blk A").await;
    let (bob_id, _bob_token) = create_profile_with_token(&harness, "me_blk_b", "Me Blk B").await;
    let bob_id_str = bob_id.as_str();

    // 1. Alice blocks Bob via @me
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/block",
        Some(json!({ "target_user_id": bob_id_str })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["type"], "blocked");

    // 2. Alice sees Bob in blocked list
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/blocked?limit=10&offset=0",
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let blocked = body.as_array().expect("blocked array");
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0]["target_user_id"], bob_id_str);

    // 3. Alice unblocks Bob
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/@me/relationships/block/{bob_id_str}"),
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 4. Blocked list is empty
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/blocked?limit=10&offset=0",
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let blocked = body.as_array().expect("blocked array");
    assert!(blocked.is_empty());
}

// ============================================
// @ME COMPLEX INTERACTION TESTS
// ============================================

#[tokio::test]
async fn test_me_privacy_blocks_friend_request_then_preset_reopens() {
    let harness = TestHarness::new().await;

    let (alice_id, alice_token) = create_profile_with_token(&harness, "me_prv_a", "Me Prv A").await;
    let (_bob_id, bob_token) = create_profile_with_token(&harness, "me_prv_b", "Me Prv B").await;
    let bob_id_str = _bob_id.as_str();

    // 1. Alice sets friend request privacy to "none"
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/privacy/friend-requests",
        Some(json!({ "allow_friend_requests_from": "none" })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 2. Bob cannot send friend request to Alice (uses admin route to simulate)
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": alice_id, "message": "hi" })),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 3. Alice applies "public" preset
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/privacy/preset",
        Some(json!({ "preset": "public" })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["allow_friend_requests_from"], "everyone");

    // 4. Now Bob can send friend request
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": alice_id, "message": "hi again" })),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 5. Alice accepts
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/relationships/request/accept",
        Some(json!({ "target_user_id": bob_id_str })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 6. Both are friends
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/@me/relationships/{bob_id_str}"),
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["type"], "friend");
}

#[tokio::test]
async fn test_me_block_prevents_friend_request() {
    let harness = TestHarness::new().await;

    let (alice_id, alice_token) =
        create_profile_with_token(&harness, "me_blkfr_a", "Me BlkFr A").await;
    let (_bob_id, bob_token) =
        create_profile_with_token(&harness, "me_blkfr_b", "Me BlkFr B").await;
    let bob_id_str = _bob_id.as_str();

    // 1. Alice blocks Bob
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/block",
        Some(json!({ "target_user_id": bob_id_str })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 2. Alice cannot send friend request to Bob (she blocked him)
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": bob_id_str, "message": "oops" })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 3. Bob cannot send friend request to Alice (she blocked him)
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": alice_id, "message": "hey" })),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_me_full_profile_update_then_verify_via_public_route() {
    let harness = TestHarness::new().await;

    let (user_id, token) = create_profile_with_token(&harness, "me_cross_a", "Me Cross A").await;

    // 1. Update profile via @me
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PATCH,
        "/api/v1/users/@me",
        Some(json!({
            "display_name": "Cross-Verified",
            "bio": "Updated via @me"
        })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 2. Set status via @me
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/status",
        Some(json!({ "status": "dnd" })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. Set custom status via @me
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/custom-status",
        Some(json!({ "text": "Busy coding", "emoji": "computer" })),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 4. Verify all changes via the public :user_id route (no auth needed)
    let (status, body) = make_json_request(
        harness.router.clone(),
        Method::GET,
        &format!("/api/v1/users/{user_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["display_name"], "Cross-Verified");
    assert_eq!(body["bio"], "Updated via @me");
    assert_eq!(body["status"], "dnd");
    assert_eq!(body["custom_status"]["text"], "Busy coding");
    assert_eq!(body["custom_status"]["emoji"], "computer");
}

#[tokio::test]
async fn test_me_three_user_social_graph() {
    let harness = TestHarness::new().await;

    // Create 3 users
    let (alice_id, alice_token) = create_profile_with_token(&harness, "me_soc_a", "Me Soc A").await;
    let (bob_id, bob_token) = create_profile_with_token(&harness, "me_soc_b", "Me Soc B").await;
    let (charlie_id, charlie_token) =
        create_profile_with_token(&harness, "me_soc_c", "Me Soc C").await;

    // Alice -> Bob: friend request, accepted
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": &bob_id, "message": "" })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/relationships/request/accept",
        Some(json!({ "target_user_id": &alice_id })),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Alice -> Charlie: friend request, accepted
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": &charlie_id, "message": "" })),
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        "/api/v1/users/@me/relationships/request/accept",
        Some(json!({ "target_user_id": &alice_id })),
        &charlie_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Alice has 2 friends
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/friends?limit=10&offset=0",
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let friends = body.as_array().expect("friends");
    assert_eq!(friends.len(), 2);

    // Bob blocks Charlie
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/block",
        Some(json!({ "target_user_id": &charlie_id })),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Bob still has Alice as friend (1 friend), and Charlie in blocked (1 blocked)
    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/friends?limit=10&offset=0",
        None,
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let bob_friends = body.as_array().expect("bob friends");
    assert_eq!(bob_friends.len(), 1);

    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/blocked?limit=10&offset=0",
        None,
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let bob_blocked = body.as_array().expect("bob blocked");
    assert_eq!(bob_blocked.len(), 1);
    assert_eq!(bob_blocked[0]["target_user_id"], charlie_id);

    // Charlie cannot send friend request to Bob (blocked)
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/users/@me/relationships/request",
        Some(json!({ "target_user_id": &bob_id, "message": "" })),
        &charlie_token,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Alice removes Bob, then her friend count drops to 1
    let (status, _) = make_authenticated_request(
        harness.router.clone(),
        Method::DELETE,
        &format!("/api/v1/users/@me/relationships/friend/{bob_id}"),
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = make_authenticated_request(
        harness.router.clone(),
        Method::GET,
        "/api/v1/users/@me/relationships/friends?limit=10&offset=0",
        None,
        &alice_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let alice_friends = body.as_array().expect("alice friends after remove");
    assert_eq!(alice_friends.len(), 1);
    assert_eq!(alice_friends[0]["target_user_id"], charlie_id);
}
