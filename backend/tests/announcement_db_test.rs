//! End-to-end coverage for the system banner announcement API (Issue #1136):
//! admin CRUD over `/api/admin/announcements`, the public
//! `/api/announcements` view, and the WebSocket broadcast emitted for every
//! mutation.
//!
//! Set `WATCHDOG_TEST_DATABASE_URL` to a throwaway PostgreSQL database to run
//! these; without it they skip, matching `inactivity_watchdog_db_test.rs` and
//! `db_query_guards_test.rs`.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::{create_router, AppState, PlanCache};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt; // for oneshot

/// Connects (and migrates) the throwaway database, or `None` when the test
/// environment did not provide one.
async fn pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("WATCHDOG_TEST_DATABASE_URL").ok()?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("WATCHDOG_TEST_DATABASE_URL is set but unreachable");
    inheritx_backend::DbManager::run_migrations(&pool)
        .await
        .expect("migrations must apply to the test database");
    Some(pool)
}

fn admin_token() -> String {
    std::env::set_var("JWT_SECRET", "announcement-db-test-secret");
    let claims = inheritx_backend::auth::Claims {
        sub: "admin-db-test".to_string(),
        role: "admin".to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"announcement-db-test-secret"),
    )
    .expect("token must sign")
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    serde_json::from_slice(&bytes).expect("body is JSON")
}

fn admin_request(method: &str, uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .header(axum::http::header::USER_AGENT, "InheritX-Client/1.0")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn announcement_crud_round_trip_broadcasts_over_websocket() {
    let Some(db_pool) = pool().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let (announcement_tx, mut announcement_rx) = tokio::sync::broadcast::channel(16);
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        plan_statistics_cache_ttl_secs: 60,
        apy_cache: dashmap::DashMap::new(),
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        status_tx: tokio::sync::broadcast::channel(16).0,
        announcement_tx,
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });
    let app = create_router(state);
    let token = admin_token();

    // ── CREATE ────────────────────────────────────────────────────────────
    let response = app
        .clone()
        .oneshot(admin_request(
            "POST",
            "/api/admin/announcements",
            &token,
            json!({
                "title": "Scheduled maintenance",
                "message": "The testnet UI is down for 30 minutes.",
                "severity": "warning"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let id = created["data"]["id"]
        .as_str()
        .expect("id returned")
        .to_string();
    assert_eq!(created["data"]["severity"], "warning");
    assert_eq!(created["data"]["active"], true);
    assert_eq!(created["data"]["created_by"], "admin-db-test");

    let event = announcement_rx.try_recv().expect("create is broadcast");
    assert_eq!(event.event_type, "ANNOUNCEMENT_CREATED");
    assert_eq!(event.announcement.id.to_string(), id);

    // ── LIST (admin) ──────────────────────────────────────────────────────
    let response = app
        .clone()
        .oneshot(admin_request(
            "GET",
            "/api/admin/announcements",
            &token,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let listed = body_json(response).await;
    assert!(listed["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["id"] == id.as_str()));

    // ── PUBLIC VIEW (active + inside schedule) ────────────────────────────
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/announcements")
                .header(axum::http::header::USER_AGENT, "InheritX-Client/1.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let public = body_json(response).await;
    assert!(public["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["id"] == id.as_str()));

    // ── UPDATE ────────────────────────────────────────────────────────────
    let response = app
        .clone()
        .oneshot(admin_request(
            "PUT",
            &format!("/api/admin/announcements/{id}"),
            &token,
            json!({ "severity": "critical", "active": true }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated = body_json(response).await;
    assert_eq!(updated["data"]["severity"], "critical");
    // Omitted fields keep their previous value.
    assert_eq!(updated["data"]["title"], "Scheduled maintenance");

    let event = announcement_rx.try_recv().expect("update is broadcast");
    assert_eq!(event.event_type, "ANNOUNCEMENT_UPDATED");
    assert_eq!(event.announcement.severity, "critical");

    // ── VALIDATION THROUGH THE FULL STACK ─────────────────────────────────
    let response = app
        .clone()
        .oneshot(admin_request(
            "POST",
            "/api/admin/announcements",
            &token,
            json!({ "title": "", "message": "nope" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // ── GET BY ID (unknown → 404) ─────────────────────────────────────────
    let missing = uuid::Uuid::new_v4();
    let response = app
        .clone()
        .oneshot(admin_request(
            "GET",
            &format!("/api/admin/announcements/{missing}"),
            &token,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // ── DELETE ────────────────────────────────────────────────────────────
    let response = app
        .clone()
        .oneshot(admin_request(
            "DELETE",
            &format!("/api/admin/announcements/{id}"),
            &token,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let deleted = body_json(response).await;
    assert_eq!(deleted["data"]["id"], id.as_str());

    let event = announcement_rx.try_recv().expect("delete is broadcast");
    assert_eq!(event.event_type, "ANNOUNCEMENT_DELETED");
    assert_eq!(event.announcement.id.to_string(), id);

    // ── GONE FROM THE PUBLIC VIEW ─────────────────────────────────────────
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/announcements")
                .header(axum::http::header::USER_AGENT, "InheritX-Client/1.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let public = body_json(response).await;
    assert!(!public["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["id"] == id.as_str()));

    // Deleting again → 404.
    let response = app
        .oneshot(admin_request(
            "DELETE",
            &format!("/api/admin/announcements/{id}"),
            &token,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn scheduled_announcements_are_hidden_from_the_public_view() {
    let Some(db_pool) = pool().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool,
        kyc_webhook_secret: None,
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
    let app = create_router(state);
    let token = admin_token();

    // Starts in one hour → visible to admins, not to the public.
    let starts_in_one_hour = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let response = app
        .clone()
        .oneshot(admin_request(
            "POST",
            "/api/admin/announcements",
            &token,
            json!({
                "title": "Future maintenance",
                "message": "Starts in an hour.",
                "severity": "info",
                "starts_at": starts_in_one_hour,
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let id = created["data"]["id"]
        .as_str()
        .expect("id returned")
        .to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/announcements")
                .header(axum::http::header::USER_AGENT, "InheritX-Client/1.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let public = body_json(response).await;
    assert!(
        !public["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == id.as_str()),
        "a banner scheduled for the future must not be public yet"
    );

    // Cleanup so repeated local runs stay deterministic.
    let response = app
        .oneshot(admin_request(
            "DELETE",
            &format!("/api/admin/announcements/{id}"),
            &token,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
