/*
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::str::FromStr;
use tower::ServiceExt;
use uuid::Uuid;

/// Generate a JWT token for a test user
fn generate_user_token(user_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: "testuser@inheritx.test".to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .unwrap()
}

/// Generate a JWT token for a test admin
fn generate_admin_token(admin_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = inheritx_backend::auth::AdminClaims {
        admin_id,
        email: "admin@inheritx.test".to_string(),
        role: "admin".to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .unwrap()
}

#[tokio::test]
async fn test_create_loan_lifecycle_success() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "user_id": user_id,
                        "plan_id": null,
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "1000.00",
                        "interest_rate_bps": 800,  // 8%
                        "collateral_amount": "0.5",
                        "due_date": due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");
    assert!(json["data"]["id"].is_string());
    assert_eq!(json["data"]["user_id"], user_id.to_string());
    assert_eq!(json["data"]["status"], "active");
    assert_eq!(json["data"]["principal"], "1000.00");
    assert_eq!(json["data"]["interestRateBps"], 800);
    assert_eq!(json["data"]["borrowAsset"], "USDC");
    assert_eq!(json["data"]["collateralAsset"], "ETH");
}

#[tokio::test]
async fn test_get_loan_lifecycle_success() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create a loan
    let create_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "500.00",
                        "interest_rate_bps": 500,
                        "collateral_amount": "0.25",
                        "due_date": due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let create_json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    let loan_id = Uuid::from_str(create_json["data"]["id"].as_str().unwrap()).unwrap();

    // Retrieve the loan
    let get_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/loans/lifecycle/{loan_id}"))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(get_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["data"]["id"], loan_id.to_string());
    assert_eq!(json["data"]["status"], "active");
    assert_eq!(json["data"]["principal"], "500.00");
}

#[tokio::test]
async fn test_list_loans_by_status() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create multiple loans
    for _ in 0..3 {
        test_context
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/loans/lifecycle")
                    .header("Authorization", format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        json!({
                            "borrow_asset": "USDC",
                            "collateral_asset": "BTC",
                            "principal": "1000.00",
                            "interest_rate_bps": 800,
                            "collateral_amount": "0.05",
                            "due_date": due_date.to_rfc3339()
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("request failed");
    }

    // List active loans
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/loans/lifecycle?status=active")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");
    assert!(json["count"].as_i64().unwrap() >= 3);
}

#[tokio::test]
async fn test_lifecycle_summary() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create a loan
    test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "1000.00",
                        "interest_rate_bps": 800,
                        "collateral_amount": "0.5",
                        "due_date": due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    // Get summary
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/loans/lifecycle/summary")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert!(json["data"]["total"].as_i64().unwrap() >= 1);
    assert!(json["data"]["active"].as_i64().unwrap() >= 1);
    assert_eq!(json["data"]["repaid"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_repay_loan_partial() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create a loan
    let create_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "1000.00",
                        "interest_rate_bps": 800,
                        "collateral_amount": "0.5",
                        "due_date": due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let create_json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    let loan_id = Uuid::from_str(create_json["data"]["id"].as_str().unwrap()).unwrap();

    // Make a partial repayment
    let repay_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/loans/lifecycle/{loan_id}/repay"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"amount": "300.00"}).to_string()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(repay_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(repay_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // Should still be active after partial repayment
    assert_eq!(json["data"]["status"], "active");
    assert_eq!(json["data"]["amountRepaid"], "300.00");
}

#[tokio::test]
async fn test_repay_loan_full() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create a loan
    let create_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "500.00",
                        "interest_rate_bps": 500,
                        "collateral_amount": "0.25",
                        "due_date": due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let create_json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    let loan_id = Uuid::from_str(create_json["data"]["id"].as_str().unwrap()).unwrap();

    // Full repayment
    let repay_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/loans/lifecycle/{loan_id}/repay"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"amount": "500.00"}).to_string()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(repay_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(repay_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // Should transition to repaid
    assert_eq!(json["data"]["status"], "repaid");
    assert_eq!(json["data"]["amountRepaid"], "500.00");
    assert!(json["data"]["repaidAt"].is_string());
}

#[tokio::test]
async fn test_liquidate_loan_as_admin() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let user_token = generate_user_token(user_id);
    let admin_id = Uuid::new_v4();
    let admin_token = generate_admin_token(admin_id);
    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create a loan as user
    let create_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {user_token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "1000.00",
                        "interest_rate_bps": 800,
                        "collateral_amount": "0.5",
                        "due_date": due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let create_json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    let loan_id = Uuid::from_str(create_json["data"]["id"].as_str().unwrap()).unwrap();

    // Liquidate as admin
    let liquidate_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/loans/lifecycle/{loan_id}/liquidate"))
                .header("Authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(liquidate_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(liquidate_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["data"]["status"], "liquidated");
    assert!(json["data"]["liquidatedAt"].is_string());
}

#[tokio::test]
async fn test_mark_overdue_loans() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let user_token = generate_user_token(user_id);
    let admin_id = Uuid::new_v4();
    let admin_token = generate_admin_token(admin_id);

    // Create a loan with a due date in the past
    let past_due_date = chrono::Utc::now() - chrono::Duration::hours(1);

    test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {user_token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "1000.00",
                        "interest_rate_bps": 800,
                        "collateral_amount": "0.5",
                        "due_date": past_due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    // Trigger the mark-overdue sweep
    let mark_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/loans/lifecycle/mark-overdue")
                .header("Authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(mark_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(mark_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // Should have marked at least one loan as overdue
    assert!(json["marked_overdue"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_cannot_repay_already_repaid_loan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let due_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create and fully repay a loan
    let create_response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/loans/lifecycle")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "borrow_asset": "USDC",
                        "collateral_asset": "ETH",
                        "principal": "100.00",
                        "interest_rate_bps": 500,
                        "collateral_amount": "0.05",
                        "due_date": due_date.to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let create_json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    let loan_id = Uuid::from_str(create_json["data"]["id"].as_str().unwrap()).unwrap();

    // Full repayment
    test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/loans/lifecycle/{loan_id}/repay"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"amount": "100.00"}).to_string()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    // Try to repay again - should fail
    let fail_repay = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/loans/lifecycle/{loan_id}/repay"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"amount": "50.00"}).to_string()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    // Should get a bad request error
    assert_eq!(fail_repay.status(), StatusCode::BAD_REQUEST);
}
*/
