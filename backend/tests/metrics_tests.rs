//! Integration tests for Prometheus metrics middleware (Issue #940).
//!
//! Verifies that request count and latency are recorded and exposed on `/metrics`.

#![cfg(feature = "metrics")]

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use inheritx_backend::{create_router, metrics, AppState, PlanCache};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

fn setup_app() -> axum::Router {
    metrics::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/test".to_string());

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy(&database_url)
        .unwrap();

    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool,
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });
    create_router(state)
}

async fn metrics_body(app: axum::Router) -> String {
    let response = app
        .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(
        content_type.contains("text/plain"),
        "expected Prometheus text content-type, got {content_type}"
    );

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn metrics_endpoint_exposes_prometheus_text() {
    let body = metrics_body(setup_app()).await;

    assert!(
        body.contains("inheritx_http_requests_total")
            || body.contains("# HELP inheritx_active_connections"),
        "expected Prometheus metric families in /metrics output"
    );
}

#[tokio::test]
async fn metrics_records_request_count_and_latency() {
    let app = setup_app();

    // Hit a public route (no auth) so middleware records count + latency.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/kyc/required")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = metrics_body(app).await;

    assert!(
        body.contains("inheritx_http_requests_total"),
        "missing request count metric:\n{body}"
    );
    assert!(
        body.contains("inheritx_http_request_duration_seconds"),
        "missing latency histogram:\n{body}"
    );
    assert!(
        body.contains("/api/kyc/required"),
        "expected matched path label in metrics:\n{body}"
    );
    assert!(
        body.contains("method=\"GET\"") || body.contains("method=\"get\""),
        "expected method label in metrics:\n{body}"
    );
}
