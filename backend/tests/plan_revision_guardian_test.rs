//! Database-backed coverage for the plan audit trail (issue #1133) and the
//! guardian signature aggregator (issue #1134).
//!
//! Set `WATCHDOG_TEST_DATABASE_URL` to a throwaway PostgreSQL database to run
//! these; without it they skip, matching the other database-backed suites.

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use inheritx_backend::{create_router, AppState, PlanCache};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn env_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

async fn database() -> Option<PgPool> {
    let url = std::env::var("WATCHDOG_TEST_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url)
        .await
        .expect("WATCHDOG_TEST_DATABASE_URL is set but unreachable");

    inheritx_backend::DbManager::run_migrations(&pool)
        .await
        .expect("migrations must apply cleanly");

    Some(pool)
}

fn app(db: PgPool) -> axum::Router {
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool: db,
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        status_tx: tokio::sync::broadcast::channel(16).0,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        plan_statistics_cache_ttl_secs: 60,
        apy_cache: dashmap::DashMap::new(),
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });

    create_router(state)
}

/// Signs the canonical `"{timestamp}.{body}"` payload expected by the
/// signature-auth middleware.
fn signed_headers_at(body: &str, timestamp: i64) -> (String, String, String) {
    let secret_bytes: [u8; 32] = [
        0x9d, 0x61, 0xb8, 0xbb, 0xd0, 0xa3, 0x0a, 0x78, 0x23, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
        0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
        0xde, 0xf0,
    ];
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let public_key = format!("0x{}", hex::encode(signing_key.verifying_key().to_bytes()));

    let canonical = inheritx_backend::auth::canonical_signed_payload(timestamp, body);
    let signature = signing_key.sign(canonical.as_bytes());

    (
        public_key,
        hex::encode(signature.to_bytes()),
        timestamp.to_string(),
    )
}

fn signed_headers(body: &str) -> (String, String, String) {
    signed_headers_at(body, Utc::now().timestamp() + 1)
}

async fn insert_plan(db: &PgPool, owner: &str, onchain_plan_id: Option<i64>) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO plans (
            owner_address, token_address, amount, grace_period,
            grace_period_seconds, last_ping, is_active, status, onchain_plan_id
        )
        VALUES ($1, 'TOKEN', 1000, 60, 60, $2, true, 'ACTIVE', $3)
        RETURNING id
        "#,
    )
    .bind(owner)
    .bind(Utc::now().timestamp())
    .bind(onchain_plan_id)
    .fetch_one(db)
    .await
    .expect("insert plan")
}

async fn cleanup(db: &PgPool, owner: &str) {
    sqlx::query("DELETE FROM plans WHERE owner_address = $1")
        .bind(owner)
        .execute(db)
        .await
        .expect("cleanup");
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn plan_updates_append_revisions_and_expose_them() {
    let Some(db) = database().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = "TEST_REVISION_OWNER";
    cleanup(&db, owner).await;
    let plan_id = insert_plan(&db, owner, None).await;

    sqlx::query(
        "INSERT INTO beneficiaries (plan_id, wallet_address, allocation_bps, fiat_anchor_info) VALUES ($1, 'OLD_BENEFICIARY', 10000, 'old-anchor')",
    )
    .bind(plan_id)
    .execute(&db)
    .await
    .expect("insert beneficiary");

    let app = app(db.clone());
    let update_body = json!({
        "beneficiaries": [{
            "address": "NEW_BENEFICIARY",
            "name": "New beneficiary",
            "allocation_bps": 10000,
            "fiat_anchor_info": "new-anchor"
        }],
        "grace_period": 120
    })
    .to_string();
    let (public_key, signature, timestamp) = signed_headers(&update_body);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::PUT)
                .uri(format!("/api/plans/{plan_id}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header("X-Public-Key", &public_key)
                .header("X-Signature", &signature)
                .header("X-Timestamp", &timestamp)
                .body(Body::from(update_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/plans/{plan_id}/revisions"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["count"], 1);
    assert_eq!(body["revisions"][0]["previous_values"]["grace_period"], 60);
    assert_eq!(
        body["revisions"][0]["previous_values"]["beneficiaries"][0]["address"],
        "OLD_BENEFICIARY"
    );
    assert_eq!(body["revisions"][0]["new_values"]["grace_period"], 120);
    assert_eq!(body["revisions"][0]["updated_by"], public_key);

    let changed = body["revisions"][0]["changed_fields"].as_array().unwrap();
    assert!(changed.iter().any(|field| field == "grace_period"));
    assert!(changed.iter().any(|field| field == "beneficiaries"));

    // Re-submitting the same values must not create an empty revision.
    let (_, second_signature, second_timestamp) =
        signed_headers_at(&update_body, Utc::now().timestamp() + 2);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::PUT)
                .uri(format!("/api/plans/{plan_id}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header("X-Public-Key", &public_key)
                .header("X-Signature", &second_signature)
                .header("X-Timestamp", &second_timestamp)
                .body(Body::from(update_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/plans/{plan_id}/revisions"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let revisions = response_json(response).await;
    assert_eq!(revisions["count"], 1);

    cleanup(&db, owner).await;
}

#[tokio::test]
async fn guardian_signatures_are_deduplicated_and_report_threshold() {
    let Some(db) = database().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let _guard = env_lock().lock().await;
    std::env::set_var("GUARDIAN_UNLOCK_THRESHOLD", "2");
    std::env::remove_var("GUARDIAN_UNLOCK_GUARDIANS");

    let owner = "TEST_GUARDIAN_OWNER";
    cleanup(&db, owner).await;
    let onchain_plan_id = (Uuid::new_v4().as_u128() & 0x7fff_ffff_ffff_ffff) as i64;
    let plan_id = insert_plan(&db, owner, Some(onchain_plan_id)).await;

    let app = app(db.clone());
    let timestamp = Utc::now().timestamp();
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let guardian =
        stellar_strkey::ed25519::PublicKey(signing_key.verifying_key().to_bytes()).to_string();
    let message = inheritx_backend::api::guardian_unlock_message(&plan_id, &guardian, timestamp);
    let signature = hex::encode(signing_key.sign(message.as_bytes()).to_bytes());

    let payload = json!({
        "guardian": guardian,
        "signature": signature,
        "timestamp": timestamp,
    })
    .to_string();

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("/api/plans/{plan_id}/guardian-sign"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    assert_eq!(first_body["status"], "pending");
    assert_eq!(first_body["signatures_collected"], 1);
    assert_eq!(first_body["threshold"], 2);
    assert_eq!(first_body["threshold_reached"], false);

    let duplicate = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("/api/plans/{plan_id}/guardian-sign"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate.status(), StatusCode::OK);
    let duplicate_body = response_json(duplicate).await;
    assert_eq!(duplicate_body["already_signed"], true);
    assert_eq!(duplicate_body["signatures_collected"], 1);

    // A second distinct guardian reaches the quorum. Soroban is intentionally
    // not configured in this test, so submission reports that it is pending
    // rather than pretending the funds were unlocked.
    let second_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let second_guardian =
        stellar_strkey::ed25519::PublicKey(second_key.verifying_key().to_bytes()).to_string();
    let second_message =
        inheritx_backend::api::guardian_unlock_message(&plan_id, &second_guardian, timestamp);
    let second_signature = hex::encode(second_key.sign(second_message.as_bytes()).to_bytes());

    let second = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("/api/plans/{plan_id}/guardian-sign"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "guardian": second_guardian,
                        "signature": second_signature,
                        "timestamp": timestamp,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::SERVICE_UNAVAILABLE);
    let second_body = response_json(second).await;
    assert_eq!(second_body["threshold_reached"], true);
    assert_eq!(second_body["signatures_collected"], 2);

    let stored: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM guardian_signatures WHERE plan_id = $1")
            .bind(plan_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(stored, 2);

    std::env::remove_var("GUARDIAN_UNLOCK_THRESHOLD");
    cleanup(&db, owner).await;
}
