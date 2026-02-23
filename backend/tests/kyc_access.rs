// Integration tests for KYC-protected endpoints
mod helpers;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use helpers::TestContext;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn kyc_pending_forbidden() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };
    // Create a user with pending KYC (default)
    let user_id = Uuid::new_v4();
    // No KYC approval or rejection, so status is pending
    // Try to access a protected endpoint (e.g., create_plan)
    let req = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Test Plan",
                "net_amount": 100,
                "fee": 2
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn kyc_rejected_forbidden() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };
    let user_id = Uuid::new_v4();
    // Set KYC to rejected
    let admin_id = Uuid::new_v4();
    let req_reject = Request::builder()
        .method("POST")
        .uri("/api/admin/kyc/reject")
        .header("Content-Type", "application/json")
        .header("X-Admin-Id", admin_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({ "user_id": user_id })).unwrap(),
        ))
        .unwrap();
    let _ = ctx
        .app
        .clone()
        .oneshot(req_reject)
        .await
        .expect("reject failed");
    // Try to access a protected endpoint
    let req = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Test Plan",
                "net_amount": 100,
                "fee": 2
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn kyc_approved_success() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };
    let user_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    // Approve KYC
    let req_approve = Request::builder()
        .method("POST")
        .uri("/api/admin/kyc/approve")
        .header("Content-Type", "application/json")
        .header("X-Admin-Id", admin_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({ "user_id": user_id })).unwrap(),
        ))
        .unwrap();
    let _ = ctx
        .app
        .clone()
        .oneshot(req_approve)
        .await
        .expect("approve failed");
    // Try to access a protected endpoint
    let req = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Test Plan",
                "net_amount": 100,
                "fee": 2
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);
}
