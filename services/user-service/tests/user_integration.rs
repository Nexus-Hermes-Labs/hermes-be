mod common;

use axum::http::{Method, StatusCode};
use common::helpers::make_json_request;
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

// ============================================
// PROFILE MANAGEMENT TESTS
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
// CHANGE USERNAME TESTS
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
// DELETE PROFILE TESTS
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
// STATUS & PRESENCE TESTS
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

        assert_eq!(status, StatusCode::OK, "setting {status_val} failed: {body}");
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
// SEARCH & DISCOVERY TESTS
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
    assert_eq!(body["available"], true);

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
    assert_eq!(body["available"], false);
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
// RELATIONSHIP TESTS
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
    assert_eq!(status, StatusCode::FORBIDDEN);
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
// PRIVACY TESTS
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
// COMPLEX INTERACTION TESTS
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
    // TODO: bu kismi bi dusunelim. blocklamalar tek tarafli mi yoksa cift tarafli mi olmali?
    assert_eq!(status, StatusCode::NOT_FOUND);
}
