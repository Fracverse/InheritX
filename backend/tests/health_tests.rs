mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::{create_app, Config};
use serde_json::Value;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn health_db_returns_200_when_database_connected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/health/db")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /health/db failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_db_returns_500_when_database_is_unavailable() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    test_context.pool.close().await;

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/health/db")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /health/db failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_health_endpoint() {
    // Setup configuration with dummy values for testing
    let config = Config {
        database_url: "postgres://localhost/unused".to_string(),
        port: 0,
        jwt_secret: "test_secret".to_string(),
    };

    // Create a lazy connection pool
    let db_pool =
        sqlx::PgPool::connect_lazy(&config.database_url).expect("Failed to create lazy db pool");

    // Create the application using the actual router
    let app = create_app(db_pool, config)
        .await
        .expect("Failed to create app");

    // Bind to a local address
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get addr");

    // Spawn server with ConnectInfo enabled.
    // This is required for tower-governor (rate limiting) to find the client IP.
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("Server failed");
    });

    // Client
    let client = reqwest::Client::new();
    let url = format!("http://{}/health", addr);

    // Give it a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let response = client.get(&url).send().await.expect("Request failed");

    let status = response.status();
    let body_text = response.text().await.expect("Failed to read body");

    assert_eq!(
        status,
        reqwest::StatusCode::OK,
        "Status was {}, body: {}",
        status,
        body_text
    );

    let body: Value = serde_json::from_str(&body_text).expect("Failed to parse JSON");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["message"], "App is healthy");
}
