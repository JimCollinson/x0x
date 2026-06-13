use reqwest::StatusCode;
use serde_json::Value;

#[path = "harness/src/cluster.rs"]
mod cluster;

const LAST_ADMIN_REST_ERROR: &str =
    "a group must always have at least one admin; make another member an admin first";

async fn create_public_group(daemon: &cluster::AgentInstance) -> String {
    let response = daemon
        .post(
            "/groups",
            serde_json::json!({
                "name": "last-admin rest",
                "description": "last-admin exact error regression",
                "preset": "public_open",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = response.json().await.expect("create group json");
    assert_eq!(body["ok"], true, "create group response: {body:?}");
    body["group_id"].as_str().expect("group_id").to_string()
}

async fn assert_last_admin_conflict(response: reqwest::Response) {
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body: Value = response.json().await.expect("last-admin error json");
    assert_eq!(body, serde_json::json!({ "error": LAST_ADMIN_REST_ERROR }));
}

#[tokio::test]
async fn last_admin_rest_precheck_exact_string_for_remove_ban_and_demote() {
    let (daemon, _bind_port) = cluster::solo().await;
    let group_id = create_public_group(&daemon).await;
    let agent_id = daemon.agent_id().await;

    assert_last_admin_conflict(
        daemon
            .delete(&format!("/groups/{group_id}/members/{agent_id}"))
            .await,
    )
    .await;

    assert_last_admin_conflict(
        daemon
            .post(
                &format!("/groups/{group_id}/ban/{agent_id}"),
                serde_json::json!({}),
            )
            .await,
    )
    .await;

    assert_last_admin_conflict(
        daemon
            .patch(
                &format!("/groups/{group_id}/members/{agent_id}/role"),
                serde_json::json!({ "role": "member" }),
            )
            .await,
    )
    .await;
}
