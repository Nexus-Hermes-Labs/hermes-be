#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use axum::http::{Method, StatusCode};
use common::helpers::{make_authenticated_request, make_json_request};
use common::setup::TestHarness;
use serde_json::json;
use uuid::Uuid;

// ============================================================
// CONVERSATIONS — DM
// ============================================================

#[tokio::test]
async fn open_dm_returns_200() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].is_string());
    assert_eq!(body["conversation_type"], "dm");
}

#[tokio::test]
async fn open_dm_is_idempotent() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    let (_, first) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;

    let (status, second) = make_authenticated_request(
        harness.router,
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["id"], second["id"]);
}

#[tokio::test]
async fn open_dm_without_auth_returns_401() {
    let harness = TestHarness::new().await;

    let (status, _) = make_json_request(
        harness.router,
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": Uuid::new_v4() })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_group_dm_returns_201() {
    let harness = TestHarness::new().await;
    let requester = Uuid::new_v4().to_string();
    let members = vec![Uuid::new_v4(), Uuid::new_v4()];

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        "/api/v1/conversations/group",
        Some(json!({ "member_ids": members })),
        &requester,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["id"].is_string());
    assert_eq!(body["conversation_type"], "group_dm");
}

#[tokio::test]
async fn create_group_dm_too_few_members_returns_422() {
    let harness = TestHarness::new().await;
    let requester = Uuid::new_v4().to_string();

    // member_ids requires at least 2 entries (validator min = 2)
    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        "/api/v1/conversations/group",
        Some(json!({ "member_ids": [Uuid::new_v4()] })),
        &requester,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
}

#[tokio::test]
async fn get_my_conversations_empty_returns_200() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        "/api/v1/conversations/@me",
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["conversations"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);
}

#[tokio::test]
async fn get_my_conversations_returns_created_dm() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        "/api/v1/conversations/@me",
        None,
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["conversations"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn get_conversation_returns_200() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;
    let conv_id = created["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/conversations/{conv_id}"),
        None,
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], conv_id);
    assert_eq!(body["conversation_type"], "dm");
}

#[tokio::test]
async fn get_conversation_not_found_returns_404() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let fake_id = Uuid::new_v4();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/conversations/{fake_id}"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_conversation_members_returns_both_users() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;
    let conv_id = created["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/conversations/{conv_id}/members"),
        None,
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let members = body.as_array().unwrap();
    assert_eq!(members.len(), 2);
}

#[tokio::test]
async fn add_member_to_group_dm_returns_204() {
    let harness = TestHarness::new().await;
    let requester = Uuid::new_v4().to_string();
    let initial_members = vec![Uuid::new_v4(), Uuid::new_v4()];
    let new_member = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/group",
        Some(json!({ "member_ids": initial_members })),
        &requester,
    )
    .await;
    let conv_id = created["id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/conversations/{conv_id}/members"),
        Some(json!({ "user_id": new_member })),
        &requester,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn leave_conversation_returns_204() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    let (_, created) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;
    let conv_id = created["id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/conversations/{conv_id}/members/@me"),
        None,
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ============================================================
// MESSAGES — channel
// ============================================================

#[tokio::test]
async fn send_channel_message_returns_201() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Hello, channel!" })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["id"].is_string());
    assert_eq!(body["content"], "Hello, channel!");
    assert_eq!(body["channel_id"], channel_id.to_string());
    assert_eq!(body["user_id"], user_id);
    assert!(!body["is_deleted"].as_bool().unwrap());
}

#[tokio::test]
async fn send_channel_message_empty_content_returns_422() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "" })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
}

#[tokio::test]
async fn send_channel_message_without_auth_returns_401() {
    let harness = TestHarness::new().await;
    let channel_id = Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Hello" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_channel_messages_returns_list() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    // Send two messages
    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "First message" })),
        &user_id,
    )
    .await;
    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Second message" })),
        &user_id,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/channels/{channel_id}/messages"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert!(body["has_more"].is_boolean());
}

#[tokio::test]
async fn get_channel_messages_respects_limit() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    // Send 3 messages
    for i in 0..3 {
        make_authenticated_request(
            harness.router.clone(),
            Method::POST,
            &format!("/api/v1/channels/{channel_id}/messages"),
            Some(json!({ "content": format!("Message {i}") })),
            &user_id,
        )
        .await;
    }

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/channels/{channel_id}/messages?limit=2"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
    assert_eq!(body["has_more"], true);
}

// ============================================================
// MESSAGES — conversation
// ============================================================

#[tokio::test]
async fn send_conversation_message_returns_201() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    let (_, conv) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;
    let conv_id = conv["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/conversations/{conv_id}/messages"),
        Some(json!({ "content": "Hello DM!" })),
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["content"], "Hello DM!");
    assert_eq!(body["conversation_id"], conv_id);
}

#[tokio::test]
async fn get_conversation_messages_returns_list() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4();

    let (_, conv) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        "/api/v1/conversations/dm",
        Some(json!({ "target_user_id": user_b })),
        &user_a,
    )
    .await;
    let conv_id = conv["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/conversations/{conv_id}/messages"),
        Some(json!({ "content": "msg 1" })),
        &user_a,
    )
    .await;
    make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/conversations/{conv_id}/messages"),
        Some(json!({ "content": "msg 2" })),
        &user_a,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/conversations/{conv_id}/messages"),
        None,
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
}

// ============================================================
// MESSAGES — edit & delete
// ============================================================

#[tokio::test]
async fn edit_message_returns_200() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, sent) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Original" })),
        &user_id,
    )
    .await;
    let msg_id = sent["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/messages/{msg_id}"),
        Some(json!({ "content": "Edited" })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["content"], "Edited");
    assert!(body["edited_at"].is_string());
}

#[tokio::test]
async fn edit_message_by_non_author_returns_403() {
    let harness = TestHarness::new().await;
    let author = Uuid::new_v4().to_string();
    let other = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, sent) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Original" })),
        &author,
    )
    .await;
    let msg_id = sent["id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::PATCH,
        &format!("/api/v1/messages/{msg_id}"),
        Some(json!({ "content": "Hacked" })),
        &other,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_message_returns_204() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, sent) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "To be deleted" })),
        &user_id,
    )
    .await;
    let msg_id = sent["id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{msg_id}"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_message_by_non_author_returns_403() {
    let harness = TestHarness::new().await;
    let author = Uuid::new_v4().to_string();
    let other = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, sent) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "mine" })),
        &author,
    )
    .await;
    let msg_id = sent["id"].as_str().unwrap();

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{msg_id}"),
        None,
        &other,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn send_message_with_reply_returns_201() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, original) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Original message" })),
        &user_id,
    )
    .await;
    let original_id = original["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "This is a reply", "reply_to_id": original_id })),
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["reply_to_id"], original_id);
}

// ============================================================
// REACTIONS
// ============================================================

#[tokio::test]
async fn add_reaction_returns_201() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "React to me" })),
        &user_id,
    )
    .await;
    let msg_id = msg["id"].as_str().unwrap();

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/thumbsup"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["emoji"], "thumbsup");
    assert_eq!(body["message_id"], msg_id);
    assert_eq!(body["user_id"], user_id);
}

#[tokio::test]
async fn get_reactions_returns_list() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "React to me" })),
        &user_a,
    )
    .await;
    let msg_id = msg["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/thumbsup"),
        None,
        &user_a,
    )
    .await;
    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/thumbsup"),
        None,
        &user_b,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/messages/{msg_id}/reactions"),
        None,
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn count_reactions_returns_correct_count() {
    let harness = TestHarness::new().await;
    let user_a = Uuid::new_v4().to_string();
    let user_b = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "Count reactions" })),
        &user_a,
    )
    .await;
    let msg_id = msg["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/heart"),
        None,
        &user_a,
    )
    .await;
    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/heart"),
        None,
        &user_b,
    )
    .await;

    let (status, body) = make_authenticated_request(
        harness.router,
        Method::GET,
        &format!("/api/v1/messages/{msg_id}/reactions/heart/count"),
        None,
        &user_a,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["emoji"], "heart");
    assert_eq!(body["count"], 2);
}

#[tokio::test]
async fn remove_reaction_returns_204() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "React then remove" })),
        &user_id,
    )
    .await;
    let msg_id = msg["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/wave"),
        None,
        &user_id,
    )
    .await;

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::DELETE,
        &format!("/api/v1/messages/{msg_id}/reactions/wave"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn add_duplicate_reaction_returns_conflict() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::new_v4().to_string();
    let channel_id = Uuid::new_v4();

    let (_, msg) = make_authenticated_request(
        harness.router.clone(),
        Method::POST,
        &format!("/api/v1/channels/{channel_id}/messages"),
        Some(json!({ "content": "React twice" })),
        &user_id,
    )
    .await;
    let msg_id = msg["id"].as_str().unwrap();

    make_authenticated_request(
        harness.router.clone(),
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/fire"),
        None,
        &user_id,
    )
    .await;

    let (status, _) = make_authenticated_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/fire"),
        None,
        &user_id,
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn reaction_without_auth_returns_401() {
    let harness = TestHarness::new().await;
    let msg_id = Uuid::new_v4();

    let (status, _) = make_json_request(
        harness.router,
        Method::PUT,
        &format!("/api/v1/messages/{msg_id}/reactions/thumbsup"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
