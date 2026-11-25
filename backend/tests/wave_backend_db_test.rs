//! Database-backed acceptance coverage for issues #1268, #1273 and #1274.
//!
//! Set `WATCHDOG_TEST_DATABASE_URL` to a throwaway PostgreSQL database to run
//! these; without it they skip, matching the other DB suites in this crate.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::{create_router, AppState, DbManager, PlanCache};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("WATCHDOG_TEST_DATABASE_URL").ok()?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("WATCHDOG_TEST_DATABASE_URL is set but unreachable");
    DbManager::run_migrations(&pool)
        .await
        .expect("migrations must apply to the test database");
    Some(pool)
}

fn token(sub: &str, role: &str) -> String {
    std::env::set_var("JWT_SECRET", "wave-backend-test-secret");
    let claims = inheritx_backend::auth::Claims {
        sub: sub.to_string(),
        role: role.to_string(),
        exp: (chrono::Utc::now().timestamp() + 3_600) as usize,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"wave-backend-test-secret"),
    )
    .unwrap()
}

fn app(pool: sqlx::PgPool) -> axum::Router {
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool: pool,
        kyc_webhook_secret: None,
        kyc_max_resubmit_attempts: 3,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        plan_statistics_cache_ttl_secs: 60,
        apy_cache: dashmap::DashMap::new(),
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        status_tx: tokio::sync::broadcast::channel(16).0,
        announcement_tx: tokio::sync::broadcast::channel(16).0,
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });
    create_router(state)
}

fn request(method: &str, uri: &str, body: Option<Value>, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(axum::http::header::USER_AGENT, "InheritX-Test/1.0");

    if body.is_some() {
        builder = builder.header(axum::http::header::CONTENT_TYPE, "application/json");
    }
    if let Some(token) = token {
        builder = builder.header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"));
    }

    builder
        .body(Body::from(
            body.map(|value| value.to_string()).unwrap_or_default(),
        ))
        .unwrap()
}

fn multipart_request(uri: &str, boundary: &str, body: String, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(
            axum::http::header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(axum::http::header::USER_AGENT, "InheritX-Test/1.0")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(body))
        .unwrap()
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}))
}

async fn body_bytes(response: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads")
        .to_vec()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

async fn insert_plan(pool: &sqlx::PgPool, owner: &str) -> Uuid {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        r#"
        INSERT INTO plans (
            id, owner_address, token_address, amount, grace_period, grace_period_seconds,
            last_ping, is_active, status
        )
        VALUES ($1, $2, 'USDC', 1000, 3600, 86400, $3, TRUE, 'ACTIVE')
        "#,
    )
    .bind(id)
    .bind(owner)
    .bind(now)
    .execute(pool)
    .await
    .expect("plan insert");
    id
}

async fn insert_document(
    pool: &sqlx::PgPool,
    plan_id: Uuid,
    version: i32,
    content: &[u8],
    status: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO will_documents (
            document_id, plan_id, template_used, will_hash, version, filename,
            content, is_encrypted, status
        )
        VALUES ($1, $2, 'standard', $3, $4, $5, $6, FALSE, $7)
        "#,
    )
    .bind(id)
    .bind(plan_id)
    .bind(sha256_hex(content))
    .bind(version)
    .bind(format!("will_v{version}.pdf"))
    .bind(content.to_vec())
    .bind(status)
    .execute(pool)
    .await
    .expect("document insert");
    id
}

#[tokio::test]
async fn will_document_retrieval_verification_and_event_ordering() {
    let Some(pool) = pool().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = format!("GOWNER{}", Uuid::new_v4().simple());
    let plan_id = insert_plan(&pool, &owner).await;
    let v1 = insert_document(&pool, plan_id, 1, b"version one", "superseded").await;
    let v2 = insert_document(&pool, plan_id, 2, b"version two", "active").await;

    sqlx::query(
        r#"
        INSERT INTO will_events (event_type, document_id, plan_id, vault_id, payload, created_at)
        VALUES
            ('will_created', $1, $2, 'vault-1', '{"actor":"owner"}', '2026-01-01T00:00:00Z'),
            ('will_verified', $3, $2, 'vault-1', '{}', '2026-01-02T00:00:00Z')
        "#,
    )
    .bind(v2)
    .bind(plan_id)
    .bind(v2)
    .execute(&pool)
    .await
    .expect("events insert");

    let app = app(pool.clone());
    let admin = token("admin-1", "admin");
    let owner_token = token(&owner, "user");
    let stranger = token("not-the-owner", "user");

    // List is authenticated and ordered by version.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/will/documents"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let listed = body_json(response).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 2);
    assert_eq!(listed["data"][0]["version"], 1);
    assert_eq!(listed["data"][1]["version"], 2);

    // Access control: a different user is rejected, an admin is allowed.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/will/documents"),
            None,
            Some(&stranger),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/will/documents"),
            None,
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Read a single document.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/will/documents/{v2}"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = body_json(response).await;
    assert_eq!(fetched["data"]["document_id"], v2.to_string());
    assert!(fetched["data"]["pdf_base64"].is_string());

    // Download by id and by plan version.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/will/documents/{v2}/download"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_bytes(response).await, b"version two");

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/will/documents/2/download"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_bytes(response).await, b"version two");

    // Public verification of intact content.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/will/documents/{v2}/verify"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let verified = body_json(response).await;
    assert_eq!(verified["data"]["is_valid"], true);
    assert_eq!(verified["data"]["hash_match"], true);
    assert_eq!(verified["data"]["details"]["is_active_version"], true);

    // Tampered content is reported explicitly.
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/api/will/documents/{v2}/verify/content"),
            Some(json!({ "content": "version TWO" })),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let tampered = body_json(response).await;
    assert_eq!(tampered["data"]["is_valid"], false);
    assert_eq!(tampered["data"]["hash_match"], false);
    assert!(tampered["data"]["message"]
        .as_str()
        .unwrap()
        .contains("mismatch"));

    // Matching hash.
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/api/will/documents/{v2}/verify/hash"),
            Some(json!({ "hash": sha256_hex(b"version two") })),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(body_json(response).await["data"]["hash_match"], true);

    // Missing documents return 404.
    let missing = Uuid::new_v4();
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/will/documents/{missing}"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Events are returned in chronological order for both scopes.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/will/events"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    let events = body_json(response).await;
    assert_eq!(events["data"].as_array().unwrap().len(), 2);
    assert_eq!(events["data"][0]["event_type"], "will_created");
    assert_eq!(events["data"][1]["event_type"], "will_verified");

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/will/documents/{v2}/events"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(
        body_json(response).await["data"].as_array().unwrap().len(),
        2
    );

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/will/events/stats"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    let stats = body_json(response).await;
    assert_eq!(stats["data"]["total_events"], 2);
    assert_eq!(stats["data"]["will_created_count"], 1);
    assert_eq!(stats["data"]["will_verified_count"], 1);

    // Batch verification includes both versions; v1 is superseded.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/will/verify-all"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let batch = body_json(response).await;
    assert_eq!(batch["count"], 2);
    assert_eq!(batch["valid_count"], 1);
    assert_eq!(batch["invalid_count"], 1);

    let _ = v1;
}

#[tokio::test]
async fn admin_dashboard_operations_persist_and_audit() {
    let Some(pool) = pool().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = format!("GOWNER{}", Uuid::new_v4().simple());
    let plan_id = insert_plan(&pool, &owner).await;
    let app = app(pool.clone());
    let admin = token("admin-db-1", "admin");

    // Metrics.
    let response = app
        .clone()
        .oneshot(request("GET", "/api/admin/metrics", None, Some(&admin)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let metrics = body_json(response).await;
    for key in [
        "total_revenue",
        "total_plans",
        "total_claims",
        "active_plans",
        "total_users",
    ] {
        assert!(metrics["data"].get(key).is_some(), "missing {key}");
    }

    // KYC moderation.
    sqlx::query(
        "INSERT INTO users (wallet_address, kyc_status) VALUES ($1, 'submitted'::kyc_status)",
    )
    .bind(&owner)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO kyc_records (
            wallet_address, full_name, date_of_birth, street_address, city, country, postal_code
        )
        VALUES ($1, 'Test User', '1990-01-01', '1 Main St', 'Town', 'US', '00000')
        "#,
    )
    .bind(&owner)
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/admin/kyc/{owner}"),
            None,
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["data"]["status"], "submitted");

    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/admin/kyc/approve",
            Some(json!({ "user_id": owner })),
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let approved = body_json(response).await;
    assert_eq!(approved["data"]["status"], "approved");
    assert_eq!(approved["data"]["reviewed_by"], "admin-db-1");

    // Rejection requires a reason.
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/admin/kyc/reject",
            Some(json!({ "user_id": owner })),
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/admin/kyc/reject",
            Some(json!({ "user_id": owner, "reason": "Illegible document" })),
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["data"]["status"], "rejected");

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/admin/kyc/{}", Uuid::new_v4()),
            None,
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Insurance fund dashboard and list.
    sqlx::query(
        r#"
        INSERT INTO insurance_funds (name, asset_code, balance, reserve_ratio, health_status)
        VALUES ('Primary fund', 'USDC', 250000, 1.25, 'healthy')
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            "/api/admin/insurance-fund",
            None,
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let dashboard = body_json(response).await;
    assert_eq!(dashboard["data"]["fund"]["asset_code"], "USDC");
    assert!(dashboard["data"]["metrics"]
        .get("current_balance")
        .is_some());

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            "/api/admin/insurance-funds",
            None,
            Some(&admin),
        ))
        .await
        .unwrap();
    let funds = body_json(response).await;
    let funds = funds["data"].as_array().unwrap();
    assert!(!funds.is_empty());
    assert!(funds.iter().any(|fund| fund["asset_code"] == "USDC"));

    // Emergency controls persist.
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/admin/emergency/pause",
            Some(json!({ "plan_id": plan_id, "reason": "incident" })),
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let paused: bool = sqlx::query_scalar("SELECT is_paused FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(paused);

    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/admin/emergency/risk-override",
            Some(json!({ "plan_id": plan_id, "enabled": true, "reason": "manual review" })),
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let override_enabled: bool =
        sqlx::query_scalar("SELECT risk_override_enabled FROM plans WHERE id = $1")
            .bind(plan_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(override_enabled);

    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/admin/emergency/unpause",
            Some(json!({ "plan_id": plan_id })),
            Some(&admin),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let unpaused: bool = sqlx::query_scalar("SELECT is_paused FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!unpaused);

    // Each state change was audited.
    let response = app
        .clone()
        .oneshot(request("GET", "/api/admin/logs", None, Some(&admin)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let logs = body_json(response).await;
    assert!(logs["total_count"].as_i64().unwrap() >= 3);
    let actions: Vec<&str> = logs["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry["action"].as_str())
        .collect();
    assert!(actions.contains(&"emergency.pause"));
    assert!(actions.contains(&"emergency.unpause"));
    assert!(actions.contains(&"emergency.risk_override"));
}

#[tokio::test]
async fn plan_operations_update_state_and_enforce_access() {
    let Some(pool) = pool().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = format!("GOWNER{}", Uuid::new_v4().simple());
    let plan_id = insert_plan(&pool, &owner).await;
    let app = app(pool.clone());
    let owner_token = token(&owner, "user");
    let stranger = token("stranger", "user");

    // Inactivity status reflects the persisted timer and uses client units.
    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/inactivity-status"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let status = body_json(response).await;
    assert!(status["data"]["last_ping_timestamp"].as_i64().unwrap() > 1_000_000_000_000);
    assert_eq!(status["data"]["inactivity_period_days"], 1);
    assert_eq!(status["data"]["is_claimable"], false);

    let response = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/api/plans/{plan_id}/inactivity-status"),
            None,
            Some(&stranger),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Keep-alive resets last_ping and appends a ping log.
    let before: i64 = sqlx::query_scalar("SELECT last_ping FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;

    let response = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/api/plans/{plan_id}/keep-alive"),
            Some(json!({})),
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["status"], "ok");

    let after: i64 = sqlx::query_scalar("SELECT last_ping FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(after > before);
    let pings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ping_logs WHERE plan_id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(pings, 1);

    // Trigger transitions an ACTIVE plan once, then reports a conflict.
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/api/plans/{plan_id}/trigger"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["data"]["status"], "TRIGGERED");

    let status: String = sqlx::query_scalar("SELECT status FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "TRIGGERED");

    let response = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/api/plans/{plan_id}/trigger"),
            None,
            Some(&owner_token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // The admin client now uses the canonical due-for-claim path.
    let response = app
        .oneshot(request("GET", "/api/plans/due-for-claim", None, None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn kyc_upload_persists_file_and_metadata() {
    let Some(pool) = pool().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let storage = std::env::temp_dir().join(format!("inheritx-kyc-{}", Uuid::new_v4()));
    std::env::set_var("KYC_STORAGE_DIR", &storage);
    let app = app(pool.clone());
    let admin = token("admin-kyc-1", "admin");
    let boundary = "kyc-test-boundary";
    let content = b"%PDF-1.4 kyc evidence".to_vec();

    let mut body = String::new();
    body.push_str(&format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"document_type\"\r\n\r\nid\r\n"
    ));
    body.push_str(&format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"id.pdf\"\r\nContent-Type: application/pdf\r\n\r\n"
    ));
    body.push_str(&String::from_utf8_lossy(&content));
    body.push_str(&format!("\r\n--{boundary}--\r\n"));

    let response = app
        .clone()
        .oneshot(multipart_request(
            "/api/kyc/upload",
            boundary,
            body.clone(),
            &admin,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let uploaded = body_json(response).await;
    let document_id = uploaded["document_id"].as_str().unwrap().to_string();
    let url = uploaded["url"].as_str().unwrap().to_string();
    assert_eq!(url, format!("/api/kyc/documents/{document_id}"));

    let row = sqlx::query_as::<_, (String, String, i64, String)>(
        "SELECT owner_address, content_type, size_bytes, storage_path FROM kyc_documents WHERE id = $1",
    )
    .bind(Uuid::parse_str(&document_id).unwrap())
    .fetch_one(&pool)
    .await
    .expect("metadata persists");
    assert_eq!(row.0, "admin-kyc-1");
    assert_eq!(row.1, "application/pdf");
    assert_eq!(row.2, content.len() as i64);
    assert!(std::path::Path::new(&row.3).exists());

    // The returned URL resolves to the saved bytes.
    let response = app
        .clone()
        .oneshot(request("GET", &url, None, None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_bytes(response).await, content);

    // Storage failure must not emit a success response or metadata row.
    let blocker = std::env::temp_dir().join(format!("inheritx-kyc-blocker-{}", Uuid::new_v4()));
    tokio::fs::write(&blocker, b"not a directory")
        .await
        .unwrap();
    std::env::set_var("KYC_STORAGE_DIR", &blocker);

    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kyc_documents")
        .fetch_one(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(multipart_request("/api/kyc/upload", boundary, body, &admin))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kyc_documents")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before, after);

    tokio::fs::remove_file(&blocker).await.ok();
    tokio::fs::remove_dir_all(&storage).await.ok();
}
