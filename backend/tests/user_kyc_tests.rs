mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use helpers::TestContext;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_get_user_kyc_pending() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };

    let user_id = Uuid::new_v4();

    // The current implementation seems to have a bypass for tests using X-User-Id
    // as seen in kyc_access.rs and double_claim.rs
    let req = Request::builder()
        .method("GET")
        .uri("/api/kyc")
        .header("X-User-Id", user_id.to_string())
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body["user_id"], user_id.to_string());
    assert_eq!(body["status"], "pending");
}

#[tokio::test]
async fn test_get_user_kyc_unauthorized() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };

    let req = Request::builder()
        .method("GET")
        .uri("/api/kyc")
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
