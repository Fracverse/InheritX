// ──────────────────────────────────────────────────────────────────────────────
// Legal Document Download API Tests (Issue #334)
// ──────────────────────────────────────────────────────────────────────────────

mod helpers;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

// Helper to create a test user and get auth token
async fn create_test_user(pool: &sqlx::PgPool, email: &str) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(email)
    .bind(&password_hash)
    .execute(pool)
    .await
    .unwrap();

    // Generate JWT token
    let claims = inheritx_backend::auth::UserClaims {
        user_id,
        email: email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-jwt-secret".as_bytes()),
    )
    .unwrap();

    (user_id, token)
}

#[tokio::test]
async fn test_download_will_document_success() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    // Create a plan
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Generate a will document
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/will/generate", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "owner_name": "Alice Testator",
                "owner_wallet": "GABC1234567890ABCDEF",
                "vault_id": "vault-001",
                "beneficiaries": [{
                    "name": "Bob Beneficiary",
                    "wallet_address": "GBOB1234567890ABCDEF",
                    "allocation_percent": "100.0000"
                }],
                "template": "formal"
            })
            .to_string(),
        ))
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let document_id = json["data"]["document_id"].as_str().unwrap();
    let document_id = Uuid::parse_str(document_id).unwrap();

    // Download the document
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/will/documents/{}/download", document_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify headers
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/pdf"
    );
    assert!(response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("attachment; filename="));
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-cache, no-store, must-revalidate"
    );

    // Verify PDF content
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(body.starts_with(b"%PDF-1.4"));
    assert!(body.ends_with(b"%%EOF\n"));
}

#[tokio::test]
async fn test_download_will_document_unauthorized() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (owner_id, owner_token) = create_test_user(&ctx.pool, "owner@test.com").await;
    let (_other_id, other_token) = create_test_user(&ctx.pool, "other@test.com").await;

    // Create a plan for owner
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(owner_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Generate a will document
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/will/generate", plan_id))
        .header("Authorization", format!("Bearer {}", owner_token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "owner_name": "Alice Testator",
                "owner_wallet": "GABC1234567890ABCDEF",
                "vault_id": "vault-001",
                "beneficiaries": [{
                    "name": "Bob Beneficiary",
                    "wallet_address": "GBOB1234567890ABCDEF",
                    "allocation_percent": "100.0000"
                }],
                "template": "formal"
            })
            .to_string(),
        ))
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let document_id = json["data"]["document_id"].as_str().unwrap();

    // Try to download with different user (should fail)
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/will/documents/{}/download", document_id))
        .header("Authorization", format!("Bearer {}", other_token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_download_will_document_not_found() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_user_id, token) = create_test_user(&ctx.pool, "user@test.com").await;

    let non_existent_id = Uuid::new_v4();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/will/documents/{}/download", non_existent_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_download_will_document_by_version_success() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    // Create a plan
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Generate first version
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/will/generate", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "owner_name": "Alice Testator",
                "owner_wallet": "GABC1234567890ABCDEF",
                "vault_id": "vault-001",
                "beneficiaries": [{
                    "name": "Bob Beneficiary",
                    "wallet_address": "GBOB1234567890ABCDEF",
                    "allocation_percent": "100.0000"
                }],
                "template": "simple"
            })
            .to_string(),
        ))
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Generate second version
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/will/generate", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "owner_name": "Alice Testator",
                "owner_wallet": "GABC1234567890ABCDEF",
                "vault_id": "vault-001",
                "beneficiaries": [{
                    "name": "Bob Beneficiary",
                    "wallet_address": "GBOB1234567890ABCDEF",
                    "allocation_percent": "100.0000"
                }],
                "template": "formal"
            })
            .to_string(),
        ))
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Download version 1
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/plans/{}/will/documents/1/download", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify headers
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/pdf"
    );

    // Verify PDF content
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(body.starts_with(b"%PDF-1.4"));

    // Download version 2
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/plans/{}/will/documents/2/download", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_download_will_document_by_version_not_found() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    // Create a plan
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Try to download non-existent version
    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/plans/{}/will/documents/999/download",
            plan_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_download_emits_event() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    // Create a plan
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Generate a will document
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/will/generate", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "owner_name": "Alice Testator",
                "owner_wallet": "GABC1234567890ABCDEF",
                "vault_id": "vault-001",
                "beneficiaries": [{
                    "name": "Bob Beneficiary",
                    "wallet_address": "GBOB1234567890ABCDEF",
                    "allocation_percent": "100.0000"
                }],
                "template": "formal"
            })
            .to_string(),
        ))
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let document_id = json["data"]["document_id"].as_str().unwrap();
    let document_id = Uuid::parse_str(document_id).unwrap();

    // Download the document
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/will/documents/{}/download", document_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify event was logged
    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM will_event_log WHERE document_id = $1 AND event_type = 'will_decrypted'",
    )
    .bind(document_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    assert_eq!(event_count, 1);
}

#[tokio::test]
async fn test_download_requires_authentication() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let document_id = Uuid::new_v4();

    // Try to download without authentication
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/will/documents/{}/download", document_id))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_download_multiple_versions() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    // Create a plan
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Generate 3 versions
    for template in &["simple", "formal", "us_jurisdiction"] {
        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/plans/{}/will/generate", plan_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "owner_name": "Alice Testator",
                    "owner_wallet": "GABC1234567890ABCDEF",
                    "vault_id": "vault-001",
                    "beneficiaries": [{
                        "name": "Bob Beneficiary",
                        "wallet_address": "GBOB1234567890ABCDEF",
                        "allocation_percent": "100.0000"
                    }],
                    "template": template
                })
                .to_string(),
            ))
            .unwrap();

        let response = ctx.app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Download each version and verify they're different
    let mut pdf_contents = Vec::new();
    for version in 1..=3 {
        let request = Request::builder()
            .method("GET")
            .uri(format!(
                "/api/plans/{}/will/documents/{}/download",
                plan_id, version
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = ctx.app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        pdf_contents.push(body);
    }

    // Verify all PDFs are valid but different
    for content in &pdf_contents {
        assert!(content.starts_with(b"%PDF-1.4"));
        assert!(content.ends_with(b"%%EOF\n"));
    }

    // Verify they're different from each other
    assert_ne!(pdf_contents[0], pdf_contents[1]);
    assert_ne!(pdf_contents[1], pdf_contents[2]);
    assert_ne!(pdf_contents[0], pdf_contents[2]);
}
