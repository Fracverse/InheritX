mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`

// ── Helpers ──

/// Build a request with no Authorization header.
fn unauthenticated_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap()
}

// ── POST /api/plans ──

/// Without an Authorization header, creating a plan must be rejected with 401.
#[tokio::test]
async fn post_plans_without_auth_returns_401() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(unauthenticated_request("POST", "/api/plans"))
        .await
        .expect("POST /api/plans request failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "POST /api/plans must return 401 when Authorization header is absent"
    );
}

// ── POST /api/plans/:id/claim ──

/// Without an Authorization header, claiming funds for any plan must be rejected with 401.
#[tokio::test]
async fn post_plans_claim_without_auth_returns_401() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // Use a random UUID – authentication is checked before the plan is looked up.
    let plan_id = uuid::Uuid::new_v4();
    let uri = format!("/api/plans/{plan_id}/claim");

    let response = ctx
        .app
        .oneshot(unauthenticated_request("POST", &uri))
        .await
        .expect("POST /api/plans/:id/claim request failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "POST /api/plans/:id/claim must return 401 when Authorization header is absent"
    );
}

// ── GET /api/plans/:id ──

/// Without an Authorization header, fetching a plan must be rejected with 401.
#[tokio::test]
async fn get_plan_without_auth_returns_401() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let plan_id = uuid::Uuid::new_v4();
    let uri = format!("/api/plans/{plan_id}");

    let response = ctx
        .app
        .oneshot(unauthenticated_request("GET", &uri))
        .await
        .expect("GET /api/plans/:id request failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "GET /api/plans/:id must return 401 when Authorization header is absent"
    );
}

// ── GET /api/notifications ──

/// Without an Authorization header, fetching notifications must be rejected with 401.
#[tokio::test]
async fn get_notifications_without_auth_returns_401() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(unauthenticated_request("GET", "/api/notifications"))
        .await
        .expect("GET /api/notifications request failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "GET /api/notifications must return 401 when Authorization header is absent"
    );
}

// ── GET /api/admin/logs ──

/// Without an Authorization header, accessing admin audit logs must be rejected with 401.
#[tokio::test]
async fn get_admin_logs_without_auth_returns_401() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(unauthenticated_request("GET", "/api/admin/logs"))
        .await
        .expect("GET /api/admin/logs request failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "GET /api/admin/logs must return 401 when Authorization header is absent"
    );
}

// ── Error body sanity checks ──

/// Verify that the 401 response body contains an `error` field with text "Unauthorized".
#[tokio::test]
async fn unauthorized_response_body_contains_error_message() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(unauthenticated_request("GET", "/api/notifications"))
        .await
        .expect("GET /api/notifications request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");

    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("response body is not valid JSON");

    assert_eq!(
        body["error"],
        serde_json::json!("Unauthorized"),
        "401 response must include {{\"error\": \"Unauthorized\"}}"
    );
}
