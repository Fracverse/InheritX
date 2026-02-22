mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
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
