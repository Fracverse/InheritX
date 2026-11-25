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
use std::{future::Future, str::FromStr, sync::Arc};
use tower::ServiceExt; // for oneshot

struct TestDatabase {
    name: String,
    admin_pool: sqlx::PgPool,
    pool: sqlx::PgPool,
}

impl TestDatabase {
    async fn create() -> Option<Self> {
        let url = std::env::var("WATCHDOG_TEST_DATABASE_URL").ok()?;
        let options = sqlx::postgres::PgConnectOptions::from_str(&url)
            .expect("WATCHDOG_TEST_DATABASE_URL must be a valid PostgreSQL URL");
        let admin_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_with(options.clone())
            .await
            .expect("WATCHDOG_TEST_DATABASE_URL is set but unreachable");
        let name = format!("announcement_test_{}", uuid::Uuid::new_v4().simple());

        sqlx::query(&format!("CREATE DATABASE {name}"))
            .execute(&admin_pool)
            .await
            .expect("failed to create isolated announcement test database");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options.database(&name))
            .await
            .expect("failed to connect to isolated announcement test database");

        if let Err(migration_error) = inheritx_backend::DbManager::run_migrations(&pool).await {
            pool.close().await;
            let cleanup = sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
                .execute(&admin_pool)
                .await;
            admin_pool.close().await;
            if let Err(cleanup_error) = cleanup {
                panic!(
                    "failed to apply test database migrations ({migration_error}) and clean up database ({cleanup_error})"
                );
            }
            panic!("failed to apply test database migrations: {migration_error}");
        }

        Some(Self {
            name,
            admin_pool,
            pool,
        })
    }

    async fn cleanup(self) -> Result<(), sqlx::Error> {
        self.pool.close().await;
        let result = sqlx::query(&format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&self.admin_pool)
            .await
            .map(|_| ());
        self.admin_pool.close().await;
        result
    }
}

async fn run_isolated_test<F, Fut>(test: F)
where
    F: FnOnce(sqlx::PgPool) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let Some(test_database) = TestDatabase::create().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let result = tokio::spawn(test(test_database.pool.clone())).await;
    test_database
        .cleanup()
        .await
        .expect("isolated announcement test database should be removed");

    if let Err(error) = result {
        if error.is_panic() {
            std::panic::resume_unwind(error.into_panic());
        }
        panic!("announcement test task failed: {error}");
    }
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
    run_isolated_test(announcement_crud_round_trip_with_pool).await;
}

async fn announcement_crud_round_trip_with_pool(db_pool: sqlx::PgPool) {
    let (announcement_tx, mut announcement_rx) = tokio::sync::broadcast::channel(16);
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool,
        kyc_webhook_secret: None,
        kyc_max_resubmit_attempts: 3,
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
    run_isolated_test(scheduled_announcements_are_hidden_from_the_public_view_with_pool).await;
}

async fn scheduled_announcements_are_hidden_from_the_public_view_with_pool(db_pool: sqlx::PgPool) {
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool,
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
