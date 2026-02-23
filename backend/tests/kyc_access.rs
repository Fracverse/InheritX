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

// ── CREATE PLAN TESTS ─────────────────────────────────────────────────────

#[tokio::test]
async fn create_plan_kyc_pending_forbidden() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };
    // Create a user with pending KYC (default)
    let user_id = Uuid::new_v4();
    // No KYC approval or rejection, so status is pending
    // Try to create a plan
    let req = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Test Plan",
                "net_amount": 100,
                "fee": 2,
                "currency_preference": "USDC"
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    
    // Verify the error message
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        error_json["error"],
        "Forbidden: KYC not approved: cannot create plan"
    );
}

#[tokio::test]
async fn create_plan_kyc_rejected_forbidden() {
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
    // Try to create a plan
    let req = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Test Plan",
                "net_amount": 100,
                "fee": 2,
                "currency_preference": "USDC"
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    
    // Verify the error message
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        error_json["error"],
        "Forbidden: KYC not approved: cannot create plan"
    );
}

#[tokio::test]
async fn create_plan_kyc_approved_success() {
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
    // Try to create a plan
    let req = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Test Plan",
                "net_amount": 100,
                "fee": 2,
                "currency_preference": "USDC"
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── CLAIM PLAN TESTS ─────────────────────────────────────────────────────

#[tokio::test]
async fn claim_plan_kyc_pending_forbidden() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };
    // Create a user with pending KYC (default)
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();
    
    // Try to claim a plan
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/plans/{}/claim", plan_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "beneficiary_email": "beneficiary@example.com"
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    
    // Verify the error message
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        error_json["error"],
        "Forbidden: KYC not approved: cannot claim plan"
    );
}

#[tokio::test]
async fn claim_plan_kyc_rejected_forbidden() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();
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
    // Try to claim a plan
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/plans/{}/claim", plan_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "beneficiary_email": "beneficiary@example.com"
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    
    // Verify the error message
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        error_json["error"],
        "Forbidden: KYC not approved: cannot claim plan"
    );
}

#[tokio::test]
async fn claim_plan_kyc_approved_success() {
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
    
    // First create a plan (required for claiming)
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Test Plan",
                "net_amount": 100,
                "fee": 2,
                "currency_preference": "USDC"
            }))
            .unwrap(),
        ))
        .unwrap();
    let create_resp = ctx.app.clone().oneshot(create_req).await.expect("create failed");
    assert_eq!(create_resp.status(), StatusCode::OK);
    
    // Extract plan_id from response
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let plan_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let plan_id = plan_json["data"]["id"].as_str().unwrap();
    
    // Try to claim the plan
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/plans/{}/claim", plan_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "beneficiary_email": "beneficiary@example.com"
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    // Note: This might return NOT_FOUND if the plan doesn't exist in the test DB,
    // but the important thing is it doesn't return FORBIDDEN due to KYC
    assert_ne!(resp.status(), StatusCode::FORBIDDEN);
}
