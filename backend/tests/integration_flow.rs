mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_full_lifecycle_flow() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet_address = "GABC1234567890";
    let admin_email = "admin@inheritx.com";
    let admin_password = "password123";

    // 0. Setup: Create Admin in DB
    let admin_id = Uuid::new_v4();
    let hashed_password = bcrypt::hash(admin_password, bcrypt::DEFAULT_COST).unwrap();
    sqlx::query("INSERT INTO admins (id, email, password_hash, role) VALUES ($1, $2, $3, $4)")
        .bind(admin_id)
        .bind(admin_email)
        .bind(hashed_password)
        .bind("superadmin")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 1. Get nonce
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/auth/nonce/{}", wallet_address))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let nonce = body["nonce"].as_str().unwrap();
    assert!(!nonce.is_empty());

    // 2. Login via wallet
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/wallet-login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "wallet_address": wallet_address,
                        "signature": "valid_signature"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let user_token = body["token"].as_str().unwrap();

    // 3. Submit KYC
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/submit")
                .header("Authorization", format!("Bearer {}", user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Admin approve
    // First login as admin
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": admin_email,
                        "password": admin_password
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let admin_token = body["token"].as_str().unwrap();

    // Get user id from DB
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE wallet_address = $1")
        .bind(wallet_address)
        .fetch_one(&test_context.pool)
        .await
        .unwrap();

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/kyc/approve")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "user_id": user_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Create plan
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans")
                .header("Authorization", format!("Bearer {}", user_token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Inheritance Plan",
                        "description": "A test plan",
                        "fee": "2.00",
                        "net_amount": "98.00",
                        "currency_preference": "USDC"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let plan_id = body["data"]["id"].as_str().unwrap();

    // 6. Simulate maturity
    sqlx::query(
        "UPDATE plans SET status = 'active', distribution_method = 'LumpSum', contract_created_at = $1, contract_plan_id = 1 WHERE id = $2"
    )
    .bind(chrono::Utc::now().timestamp() - 86400) // 1 day ago
    .bind(Uuid::parse_str(plan_id).unwrap())
    .execute(&test_context.pool)
    .await
    .unwrap();

    // 7. Claim
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", user_token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "beneficiary_email": "beneficiary@example.com"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 8. Verify audit logs
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let logs = body["data"].as_array().unwrap();

    // Check if there is a plan_claimed log
    let has_claim_log = logs.iter().any(|log| log["action"] == "plan_claimed");
    assert!(has_claim_log, "Claim log not found in audit logs");

    // 9. Verify notifications
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notifications")
                .header("Authorization", format!("Bearer {}", user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let notifications = body["data"].as_array().unwrap();

    // In our implementation, we create a silent notification for KYC approval
    let has_kyc_notif = notifications
        .iter()
        .any(|n| n["message"].as_str().unwrap().contains("KYC"));
    assert!(has_kyc_notif, "KYC notification not found");
}
