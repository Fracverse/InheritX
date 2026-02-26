// This file contains tests for rejection functionality.
mod helpers;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use helpers::TestContext;
// use inheritx_backend::service::ClaimStatus;
use serde_json::json;
use tower::ServiceExt; // <-- required for `.oneshot()`

#[tokio::test]
async fn reject_success() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };

    let claim_id = 1;

    let req = Request::builder()
        .method("POST")
        .uri(format!("/claims/{}/reject", claim_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn cannot_reject_twice() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };

    let claim_id = 2;

    let req1 = Request::builder()
        .method("POST")
        .uri(format!("/claims/{}/reject", claim_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap();

    let resp1 = ctx.app.clone().oneshot(req1).await.expect("request failed");
    assert_eq!(resp1.status(), StatusCode::OK);

    let req2 = Request::builder()
        .method("POST")
        .uri(format!("/claims/{}/reject", claim_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap();

    let resp2 = ctx.app.clone().oneshot(req2).await.expect("request failed");
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cannot_approve_after_reject_without_reset() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };

    let claim_id = 3;

    let req_reject = Request::builder()
        .method("POST")
        .uri(format!("/claims/{}/reject", claim_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap();

    let resp_reject = ctx
        .app
        .clone()
        .oneshot(req_reject)
        .await
        .expect("request failed");
    assert_eq!(resp_reject.status(), StatusCode::OK);

    let req_approve = Request::builder()
        .method("POST")
        .uri(format!("/claims/{}/approve", claim_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap();

    let resp_approve = ctx
        .app
        .clone()
        .oneshot(req_approve)
        .await
        .expect("request failed");
    assert_eq!(resp_approve.status(), StatusCode::BAD_REQUEST);
}
