//! Route-level coverage for the Stellar Wave backend tickets #1268, #1273,
//! #1274 and #1275. These assert that every client-targeted path is registered
//! and applies the intended authorization/validation boundary before any
//! database access.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::{create_router, AppState, PlanCache};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

fn setup_app() -> axum::Router {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:1/nonexistent".to_string());

    // Lazy pool: these tests never reach a working database.
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(250))
        .connect_lazy(&database_url)
        .unwrap();

    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool,
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        status_tx: tokio::sync::broadcast::channel(16).0,
        announcement_tx: tokio::sync::broadcast::channel(16).0,
        kyc_webhook_secret: None,
        kyc_max_resubmit_attempts: 3,
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

async fn status(app: axum::Router, request: Request<Body>) -> StatusCode {
    app.oneshot(request).await.unwrap().status()
}

fn admin_token() -> String {
    std::env::set_var("JWT_SECRET", "route-test-secret");
    let claims = inheritx_backend::auth::Claims {
        sub: "route-test-admin".to_string(),
        role: "admin".to_string(),
        exp: (chrono::Utc::now().timestamp() + 3_600) as usize,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"route-test-secret"),
    )
    .unwrap()
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

fn multipart_body(boundary: &str, parts: &[(&str, &str)]) -> Body {
    let mut body = String::new();
    for (name, value) in parts {
        body.push_str(&format!("--{boundary}\r\n"));
        body.push_str(&format!(
            "Content-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
        ));
    }
    body.push_str(&format!("--{boundary}--\r\n"));
    Body::from(body)
}

#[tokio::test]
async fn plan_client_routes_are_registered_and_require_auth() {
    let plan_id = uuid::Uuid::new_v4();
    let app = setup_app();

    for (method, uri) in [
        ("POST", format!("/api/plans/{plan_id}/trigger")),
        ("POST", format!("/api/plans/{plan_id}/keep-alive")),
        ("GET", format!("/api/plans/{plan_id}/inactivity-status")),
    ] {
        let response = status(app.clone(), request(method, &uri, None, None)).await;
        assert_eq!(
            response,
            StatusCode::UNAUTHORIZED,
            "{method} {uri} must be registered and authenticated, not 404"
        );
    }
}

#[tokio::test]
async fn admin_management_routes_require_admin_jwt() {
    let app = setup_app();
    let plan_id = uuid::Uuid::new_v4();

    let cases = [
        ("GET", "/api/admin/metrics".to_string(), None),
        ("GET", "/api/admin/logs".to_string(), None),
        ("GET", "/api/admin/kyc/GABC123".to_string(), None),
        (
            "POST",
            "/api/admin/kyc/approve".to_string(),
            Some(json!({ "user_id": "GABC123" })),
        ),
        (
            "POST",
            "/api/admin/kyc/reject".to_string(),
            Some(json!({ "user_id": "GABC123", "reason": "blurry" })),
        ),
        ("GET", "/api/admin/insurance-fund".to_string(), None),
        ("GET", "/api/admin/insurance-funds".to_string(), None),
        (
            "POST",
            "/api/admin/emergency/pause".to_string(),
            Some(json!({ "plan_id": plan_id, "reason": "incident" })),
        ),
        (
            "POST",
            "/api/admin/emergency/unpause".to_string(),
            Some(json!({ "plan_id": plan_id })),
        ),
        (
            "POST",
            "/api/admin/emergency/risk-override".to_string(),
            Some(json!({ "plan_id": plan_id, "enabled": true, "reason": "review" })),
        ),
    ];

    for (method, uri, body) in cases {
        let response = status(app.clone(), request(method, &uri, body, None)).await;
        assert_eq!(
            response,
            StatusCode::UNAUTHORIZED,
            "{method} {uri} must be registered behind admin auth"
        );
    }
}

#[tokio::test]
async fn admin_management_validates_bodies_before_touching_the_database() {
    let app = setup_app();
    let token = admin_token();

    // Missing plan_id / reasons are rejected by request validation.
    let response = status(
        app.clone(),
        request(
            "POST",
            "/api/admin/emergency/pause",
            Some(json!({ "reason": "incident" })),
            Some(&token),
        ),
    )
    .await;
    assert!(
        response == StatusCode::UNPROCESSABLE_ENTITY || response == StatusCode::BAD_REQUEST,
        "unexpected status: {response}"
    );

    let response = status(
        app.clone(),
        request(
            "POST",
            "/api/admin/kyc/reject",
            Some(json!({ "user_id": "GABC123" })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn will_document_routes_are_registered() {
    let document_id = uuid::Uuid::new_v4();
    let app = setup_app();

    // Private operations require auth rather than 404.
    for uri in [
        format!("/api/will/documents/{document_id}"),
        format!("/api/will/documents/{document_id}/download"),
        format!("/api/will/documents/{document_id}/events"),
    ] {
        let response = status(app.clone(), request("GET", &uri, None, None)).await;
        assert_eq!(
            response,
            StatusCode::UNAUTHORIZED,
            "{uri} must be registered"
        );
    }

    // Public verification routes exist and validate the path id.
    for uri in [format!("/api/will/documents/{document_id}/verify")] {
        let response = status(app.clone(), request("GET", &uri, None, None)).await;
        assert_ne!(response, StatusCode::NOT_FOUND, "{uri} must be registered");
    }

    let response = status(
        app.clone(),
        request("GET", "/api/will/documents/not-a-uuid/verify", None, None),
    )
    .await;
    assert_eq!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn kyc_upload_rejects_invalid_multipart_without_storage() {
    let app = setup_app();
    let boundary = "inheritx-test-boundary";

    // No file part.
    let response = status(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/api/kyc/upload")
            .header(
                axum::http::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .header(axum::http::header::USER_AGENT, "InheritX-Test/1.0")
            .body(multipart_body(boundary, &[("document_type", "id")]))
            .unwrap(),
    )
    .await;
    assert_eq!(response, StatusCode::BAD_REQUEST);

    // Unknown document type with a file present.
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"document_type\"\r\n\r\npassport\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"id.pdf\"\r\n\
         Content-Type: application/pdf\r\n\r\n\
         %PDF-1.4 test\r\n\
         --{boundary}--\r\n"
    );
    let response = status(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/kyc/upload")
            .header(
                axum::http::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .header(axum::http::header::USER_AGENT, "InheritX-Test/1.0")
            .body(Body::from(body))
            .unwrap(),
    )
    .await;
    assert_eq!(response, StatusCode::BAD_REQUEST);
}
