// Integration test: Prevent double claim exploit
mod helpers;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use helpers::TestContext;
use serde_json::json;
use tokio::join;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn prevent_double_claim_exploit() {
    let ctx = match TestContext::from_env().await {
        Some(ctx) => ctx,
        None => return,
    };

    // Setup: create user, approve KYC, create plan
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

    // Create plan
    let req_plan = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "Exploit Test Plan",
                "net_amount": 100,
                "fee": 2
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp_plan = ctx
        .app
        .clone()
        .oneshot(req_plan)
        .await
        .expect("plan create failed");
    assert_eq!(resp_plan.status(), StatusCode::OK);

    // FIX 1: Use axum::body::to_bytes instead of removed hyper::body::to_bytes
    let body = to_bytes(resp_plan.into_body(), usize::MAX).await.unwrap();
    let plan_id = serde_json::from_slice::<serde_json::Value>(&body).unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    // FIX 2: Parse plan_id as Uuid so sqlx can bind it to a UUID column
    let plan_uuid = Uuid::parse_str(&plan_id).expect("plan_id is not a valid UUID");

    // Prepare two simultaneous claim requests
    let claim_req = || {
        Request::builder()
            .method("POST")
            .uri(format!("/api/plans/{}/claim", plan_id))
            .header("Content-Type", "application/json")
            .header("X-User-Id", user_id.to_string())
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
    };

    // Send both claims in parallel
    let (resp1, resp2) = join!(
        ctx.app.clone().oneshot(claim_req()),
        ctx.app.clone().oneshot(claim_req())
    );
    let status1 = resp1.expect("claim1 failed").status();
    let status2 = resp2.expect("claim2 failed").status();

    // One must succeed, one must fail
    assert!(
        (status1 == StatusCode::OK && status2 != StatusCode::OK)
            || (status2 == StatusCode::OK && status1 != StatusCode::OK),
        "Exactly one claim should succeed, got: {} and {}",
        status1,
        status2
    );

    // Check only one DB update
    let plan_row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM plans WHERE id = $1 AND claimed = true")
            .bind(plan_uuid) // FIX 2 applied: bind Uuid, not &String
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(plan_row.0, 1, "Plan should be claimed only once");

    // Check only one audit log
    let audit_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM plan_logs WHERE plan_id = $1 AND action = 'claim'")
            .bind(plan_uuid) // FIX 2 applied: bind Uuid, not &String
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(audit_count.0, 1, "Only one audit log for claim");
}
